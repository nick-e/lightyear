use std::ops::DerefMut;
use std::sync::Mutex;

use bevy::prelude::{
    App, FixedUpdate, IntoSystemConfigs, IntoSystemSetConfigs, Plugin as PluginType, PostUpdate,
    PreUpdate,
};

use systems::{receive, send};

use crate::plugin::sets::{FixedUpdateSet, MainSet};
use crate::plugin::systems::replication::add_replication_send_systems;
use crate::plugin::systems::tick::increment_tick;
use crate::server::input::InputPlugin;
use crate::server::Server;
use crate::{
    ClientId, ComponentProtocol, ConnectEvent, DisconnectEvent, EntitySpawnEvent, MessageProtocol,
    Protocol, ReplicationData, ReplicationSend, ReplicationSet, SharedPlugin,
};

use super::config::ServerConfig;

mod schedules;
mod systems;

pub struct PluginConfig<P: Protocol> {
    server_config: ServerConfig,
    protocol: P,
}

// TODO: put all this in ClientConfig?
impl<P: Protocol> PluginConfig<P> {
    pub fn new(server_config: ServerConfig, protocol: P) -> Self {
        PluginConfig {
            server_config,
            protocol,
        }
    }
}
pub struct Plugin<P: Protocol> {
    // we add Mutex<Option> so that we can get ownership of the inner from an immutable reference
    // in build()
    config: Mutex<Option<PluginConfig<P>>>,
}

impl<P: Protocol> Plugin<P> {
    pub fn new(config: PluginConfig<P>) -> Self {
        Self {
            config: Mutex::new(Some(config)),
        }
    }
}

impl<P: Protocol> PluginType for Plugin<P> {
    fn build(&self, app: &mut App) {
        let mut config = self.config.lock().unwrap().deref_mut().take().unwrap();
        let server = Server::new(config.server_config.clone(), config.protocol);

        // TODO: maybe put those 2 in a ReplicationPlugin?
        add_replication_send_systems::<P, Server<P>>(app);
        // P::add_per_component_replication_send_systems::<Server<P>>(app);
        P::Components::add_per_component_replication_send_systems::<Server<P>>(app);
        P::Components::add_events::<ClientId>(app);

        P::Message::add_events::<ClientId>(app);

        app
            // PLUGINS
            .add_plugins(SharedPlugin {
                // TODO: move shared config out of server_config
                config: config.server_config.shared.clone(),
            })
            .add_plugins(InputPlugin::<P>::default())
            // RESOURCES //
            .insert_resource(server)
            .init_resource::<ReplicationData>()
            // SYSTEM SETS //
            .configure_sets(PreUpdate, MainSet::Receive)
            .configure_sets(
                PostUpdate,
                (
                    ReplicationSet::SendEntityUpdates,
                    ReplicationSet::SendComponentUpdates.after(ReplicationSet::SendEntityUpdates),
                    MainSet::Send.after(ReplicationSet::SendComponentUpdates),
                ),
            )
            // EVENTS //
            .add_event::<ConnectEvent<ClientId>>()
            .add_event::<DisconnectEvent<ClientId>>()
            .add_event::<EntitySpawnEvent<ClientId>>()
            // SYSTEMS //
            .add_systems(PreUpdate, receive::<P>.in_set(MainSet::Receive))
            // TODO: a bit of a code-smell that i have to run this here instead of in the shared plugin
            //  maybe TickManager should be a separate resource not contained in Client/Server?
            .add_systems(
                FixedUpdate,
                increment_tick::<Server<P>>.in_set(FixedUpdateSet::TickUpdate),
            )
            .add_systems(PostUpdate, send::<P>.in_set(MainSet::Send));
    }
}
