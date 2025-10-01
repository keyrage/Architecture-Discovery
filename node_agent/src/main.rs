//use std::sync::mpsc;
//use std::thread;
use linux::sys_interagator;
use node_agent::inventory_client::{AgentInfo, InventoryTransport};
use serde_json::json;
use log::*;
use structopt::StructOpt;
//use std::process::{Command, Stdio};

pub mod linux;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Silence all output
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,
    /// Timestamp (sec, ms, ns, none)
    #[structopt(short = "t", long = "timestamp")]
    ts: Option<stderrlog::Timestamp>,
    /// Site code (-s sitecode)
    #[structopt(short = "s", long = "sitecode", default_value="default")]
    sitecode: String
}

fn main() {
    let opt = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
        .show_module_names(true)
        .verbosity(if opt.verbose == 0 {3} else {opt.verbose}) //Right here I'm setting a hard default to debug but I could use the cargo # flags to set depending on if it's cargo test, run or build.
        .quiet(opt.quiet)
        .timestamp(opt.ts.unwrap_or(stderrlog::Timestamp::Millisecond))
        .init()
        .unwrap();
           
    //Set up a new onject to get specifc system information needed
    let system = sys_interagator::SystemInfo::new();

    //Sets up an agent object
    let agent: AgentInfo = AgentInfo::new(system.agent_id.clone(), opt.sitecode);

    //Setup a connection to MQTT
    let mut server = InventoryTransport::new("localhost".to_string(),9001,system.agent_id.clone());
    info!("Connecting to {}",server.url);
    match server.connect() {
        Ok(()) => {info!("Connected to {}", server.url);}//println!("Connected"),
        Err(e) => {
            error!("Cannot connect to {}. Error: {}", server.url, e);
            return
        }
    };

    //Send agent information to inform subscribers that there is a new agent
    let agent_json = serde_json::to_string(&agent).unwrap();

    let system_json_string = serde_json::to_string(&agent_json).unwrap();
    debug!("System message payload: {}",system_json_string);

    server.send_message(&"/agents".to_string(),&system_json_string).unwrap();

    //Construct topics for this agent
    let node_topic = format!("/nodes/{0}",system.agent_id); //use this to send node system information and TODO: look to use the mqtt retained flag
    debug!("MQTT Node Topic path: {}", node_topic);
    let process_topic = format!("/nodes/{0}/processes",system.agent_id);
    debug!("MQTT Process Topic path: {}", process_topic);
    let net_listening_topic = format!("/nodes/{0}/net_listening",system.agent_id);
    debug!("MQTT Network Listening Topic path: {}", net_listening_topic);
    let net_connection_topic = format!("/nodes/{0}/net_connection",system.agent_id);
    debug!("MQTT Network Connections Topic path: {}", net_connection_topic);

    //Send node information, right now this is the same as the local system but is in place to allow for remote querying later
    let node_json = serde_json::to_string(&system).unwrap();
    debug!("Node message payload: {}", node_json);
    server.send_message(&node_topic,&node_json).unwrap();

    let mut syspids = sys_interagator::Processes::new();
    for syspid in syspids.processes.iter() {
        let process_json = serde_json::to_string(&syspid).unwrap();
        debug!("Process message payload: {}", process_json);
        server.send_message(&process_topic,&process_json).unwrap();
    }

    for new_syspid in syspids.get_new_processes().iter() {
        let new_process_json = serde_json::to_string(&new_syspid).unwrap();
        debug!("New process message payload: {}", new_process_json);
        server.send_message(&process_topic,&new_process_json).unwrap();
    }

    if let Ok(listeners) = listeners::get_all() {
        for l in listeners {
            //println!("{}");
            let net_listening_json = json!({
                "correlation_id":agent.correlation_id,
                "node":system.agent_id,
                "pid":l.process.pid,
                "tcp_socket":l.socket,
            });
            let net_listening_json_string = serde_json::to_string(&net_listening_json).unwrap();
            debug!("Network listening mmessage payload: {}", net_listening_json_string);
            server.send_message(&net_listening_topic.to_string(),&net_listening_json_string).unwrap();    
        }
    }
    let system_network = sys_interagator::NetConnections::new();
    for connection in system_network.connections {
        //let connection_json = serde_json::to_string(&connection);
        let net_connection_json = json!({
            "correlation_id":agent.correlation_id,
            "node":system.agent_id,
            "source_socket": connection.0,
            "destination_socket": connection.1,
            "pid": connection.2,
        });
        let net_connection_json_string = serde_json::to_string(&net_connection_json).unwrap();
        debug!("Network connection message payload: {}", net_connection_json_string);
        server.send_message(&net_connection_topic.to_string(),&net_connection_json_string).unwrap();
    }

    //Disconnect from MQTT
    info!("Disconnecting from {}",server.url);
    server.disconnect().unwrap();

}
