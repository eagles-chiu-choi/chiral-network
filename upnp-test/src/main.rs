use clap::Parser;
use libp2p::{
    futures::StreamExt,
    identify, noise, ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, upnp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use std::error::Error;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "upnp-test")]
#[command(about = "Test UPnP NAT traversal with libp2p", long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "4001")]
    port: u16,

    /// Peer address to connect to (optional)
    #[arg(short, long)]
    connect: Option<String>,
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    upnp: upnp::tokio::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    info!("🚀 Starting UPnP NAT Traversal Test");
    info!("📝 Listening port: {}", args.port);

    // Generate a random PeerId
    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    info!("🆔 Local PeerID: {}", local_peer_id);

    // Create behaviours
    let behaviour = Behaviour {
        upnp: upnp::tokio::Behaviour::default(),
        identify: identify::Behaviour::new(identify::Config::new(
            "/upnp-test/1.0.0".to_string(),
            local_key.public(),
        )),
        ping: ping::Behaviour::new(ping::Config::new()),
    };

    // Build swarm with libp2p 0.54 API
    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| behaviour)?
        .build();

    // Listen on all interfaces
    let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", args.port)
        .parse()
        .expect("Invalid listen address");

    swarm.listen_on(listen_addr.clone())?;
    info!("👂 Listening on: {}", listen_addr);

    // Connect to peer if specified
    if let Some(addr_str) = args.connect {
        match addr_str.parse::<Multiaddr>() {
            Ok(addr) => {
                info!("🔗 Attempting to connect to: {}", addr);
                if let Err(e) = swarm.dial(addr.clone()) {
                    error!("❌ Failed to dial {}: {}", addr, e);
                } else {
                    info!("✅ Dial initiated successfully");
                }
            }
            Err(e) => {
                error!("❌ Invalid multiaddr: {}", e);
            }
        }
    }

    // Event loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("📍 New listen address: {}", address);
            }
            SwarmEvent::Behaviour(event) => match event {
                BehaviourEvent::Upnp(upnp_event) => {
                    handle_upnp_event(upnp_event);
                }
                BehaviourEvent::Identify(identify_event) => {
                    handle_identify_event(identify_event);
                }
                BehaviourEvent::Ping(ping_event) => {
                    handle_ping_event(ping_event);
                }
            },
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                info!(
                    "🤝 Connection established with peer: {} at {}",
                    peer_id,
                    endpoint.get_remote_address()
                );
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                warn!("🔌 Connection closed with peer: {} (cause: {:?})", peer_id, cause);
            }
            SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                info!(
                    "📥 Incoming connection from {} to local {}",
                    send_back_addr, local_addr
                );
            }
            SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                error!(
                    "❌ Incoming connection error from {} to local {}: {}",
                    send_back_addr, local_addr, error
                );
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                error!(
                    "❌ Outgoing connection error to {:?}: {}",
                    peer_id, error
                );
            }
            _ => {}
        }
    }
}

fn handle_upnp_event(event: upnp::Event) {
    match event {
        upnp::Event::NewExternalAddr(addr) => {
            info!("🎉 UPnP: Successfully mapped external address: {}", addr);
            info!("✅ This address can be used by other peers to connect to you!");
        }
        upnp::Event::ExpiredExternalAddr(addr) => {
            warn!("⚠️  UPnP: External address expired: {}", addr);
        }
        upnp::Event::GatewayNotFound => {
            warn!("⚠️  UPnP: No UPnP gateway found on network");
            warn!("    - Make sure your router supports UPnP/IGD");
            warn!("    - Check if UPnP is enabled in router settings");
        }
        upnp::Event::NonRoutableGateway => {
            warn!("⚠️  UPnP: Gateway is not routable");
            warn!("    - Your router may be behind another NAT (carrier-grade NAT)");
        }
    }
}

fn handle_identify_event(event: identify::Event) {
    match event {
        identify::Event::Received { peer_id, info, .. } => {
            info!("🔍 Identified peer: {}", peer_id);
            info!("   Protocol version: {}", info.protocol_version);
            info!("   Agent version: {}", info.agent_version);
            info!("   Listen addresses:");
            for addr in &info.listen_addrs {
                info!("      - {}", addr);
            }
            info!("   Observed address: {:?}", info.observed_addr);
        }
        identify::Event::Sent { peer_id, .. } => {
            info!("📤 Sent identify info to peer: {}", peer_id);
        }
        identify::Event::Pushed { peer_id, .. } => {
            info!("📤 Pushed identify info to peer: {}", peer_id);
        }
        identify::Event::Error { peer_id, error, .. } => {
            error!("❌ Identify error with peer {:?}: {}", peer_id, error);
        }
    }
}

fn handle_ping_event(event: ping::Event) {
    match event {
        ping::Event {
            peer,
            result: Ok(rtt),
            ..
        } => {
            info!("🏓 Ping to {} succeeded: RTT = {:?}", peer, rtt);
        }
        ping::Event {
            peer,
            result: Err(err),
            ..
        } => {
            warn!("❌ Ping to {} failed: {}", peer, err);
        }
    }
}

