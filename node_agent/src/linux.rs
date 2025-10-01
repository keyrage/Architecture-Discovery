pub mod sys_interagator {
    use serde::{Deserialize, Serialize};
    use gethostname::gethostname;
    use pnet::datalink::interfaces;
    use pnet::ipnetwork::IpNetwork;
    use log::debug;
    use log::error;
    use log::info;
    use log::warn;
    use std::fs::{self, File};
    use std::io::{self, BufRead,Read};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
    use std::collections::HashSet;
    use std::io::ErrorKind;
    use std::path::PathBuf;
    use uuid::Uuid;

    //This is the struct to capture information about the agent. Right now I'm working on just a local agent querying local information.
    #[derive(Serialize, Deserialize)]
    pub struct SystemInfo {
        pub agent_id: String,
        //pub correlation_id: Uuid,
        pub hostname: String,
        pub ipv4_addresses: Vec<String>,
        pub ipv6_addresses: Vec<String>,
    }

    impl SystemInfo {
        pub fn new() -> Self {
            
            let machine_id_contents = Self::get_machineid();
            let hostname_contents = Self::get_hostname();
            let ipv4_addresses = Self::get_ipaddresses("v4");
            let ipv6_addresses = Self::get_ipaddresses("v6");

            Self {agent_id : String::from(machine_id_contents),
                //correlation_id : Uuid::new_v4(),
                hostname : String::from(hostname_contents),
                ipv4_addresses : ipv4_addresses,
                ipv6_addresses : ipv6_addresses,
            }
        }
        //pub fn refresh_all() -> Self {

        //}        
       
        pub fn get_machineid() -> String {
            let mut machine_id_contents = fs::read_to_string("/etc/machine-id").expect("/etc/machine-id not found or can't be opened");
            if machine_id_contents.ends_with('\n'){
                machine_id_contents.pop();
            }
            return machine_id_contents
        }

        pub fn get_hostname() -> String {
            let result = gethostname().into_string().unwrap();
            return result
        }

        //Function which gets all ipaddresses of the specified hamily "v4" or "v6"
        pub fn get_ipaddresses(v: &str) -> Vec<String> {
            let mut result = vec![];
            let all_interfaces = interfaces();
            let interfaces = all_interfaces
                .iter()
                .find(|int| int.is_up() && !int.is_loopback() && !int.ips.is_empty());
            match interfaces {
                Some(interface) => info!("Found interface with [{}]", interface.name),
                None => warn!("Where did all the networks go?"),
            }
            for interface in &interfaces{
                for ip in &interface.ips {
                    match ip {
                        IpNetwork::V4(_) => {
                            debug!("Interface: {}, IPv4: {}",interface.name,ip);
                            if v.to_lowercase() == "v4"{result.push(ip.to_string())};
                        }
                        IpNetwork::V6(_) => {
                            debug!("Interface: {}, IPv6: {}",interface.name,ip);
                            if v.to_lowercase() == "v6"{result.push(ip.to_string())};
                        }
                    }
                    
                }
            }
            debug! ("IPv4 addresses: {:?}", result);
            return result
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct NetConnections {
        pub connections: Vec<(SocketAddr, SocketAddr,u32)>,
    }

    impl NetConnections {
        pub fn new() -> Self {
             
               Self {connections : Self::get_established_connections().unwrap(),}
            
        }
        

        fn parse_ip_port(hex_str: &str, is_ipv6: bool) -> Option<(IpAddr, u16)> {
            let parts: Vec<&str> = hex_str.split(':').collect();
            if parts.len() != 2 {
                return None;
            }
            
            let port = u16::from_str_radix(parts[1], 16).ok()?;
            
            if is_ipv6 {
                let ip_bytes: Vec<u16> = parts[0]
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(4)
                    .map(|chunk| u16::from_str_radix(&chunk.iter().collect::<String>(), 16).unwrap_or(0))
                    .collect();
                if ip_bytes.len() == 8 {
                    let ip = Ipv6Addr::new(
                        ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3],
                        ip_bytes[4], ip_bytes[5], ip_bytes[6], ip_bytes[7]
                    );
                    Some((IpAddr::V6(ip), port))
                } else {
                    None
                }
            } else {
                let ip_u32 = u32::from_str_radix(parts[0], 16).ok()?;
                let ip = Ipv4Addr::from(ip_u32.swap_bytes());
                Some((IpAddr::V4(ip), port))
            }
        }

        fn get_pid_from_inode(inode: &str) -> Option<u32> {
            let proc_dir = fs::read_dir("/proc").ok()?;
            
            for entry in proc_dir {
                if let Ok(entry) = entry {
                    if let Ok(pid) = entry.file_name().into_string() {
                        if pid.chars().all(char::is_numeric) {
                            let fd_dir = format!("/proc/{}/fd", pid);
                            if let Ok(fds) = fs::read_dir(fd_dir) {
                                for fd in fds {
                                    if let Ok(fd) = fd {
                                        if let Ok(link) = fs::read_link(fd.path()) {
                                            if let Some(socket_inode) = link.to_str() {
                                                if socket_inode.contains(inode) {
                                                    return pid.parse().ok();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            None
        }

        fn get_established_connections_from_proc(proc_file: &str, is_ipv6: bool) -> io::Result<Vec<(SocketAddr, SocketAddr,u32)>> {
            let file = File::open(proc_file)?;
            let reader = io::BufReader::new(file);
            let mut connections = Vec::new();

            for line in reader.lines().skip(1) {
                let line = line?;
                let columns: Vec<&str> = line.split_whitespace().collect();
                if columns.len() > 3 && columns[3] == "01" { // 01 means ESTABLISHED
                    if let (Some(local), Some(remote), Some(pid)) = (Self::parse_ip_port(columns[1], is_ipv6), Self::parse_ip_port(columns[2], is_ipv6), Self::get_pid_from_inode(columns[9])) {
                        connections.push((SocketAddr::new(local.0, local.1), SocketAddr::new(remote.0, remote.1),pid));
                    }
                }
            }

            Ok(connections)
        }

        pub fn get_established_connections() -> io::Result<Vec<(SocketAddr, SocketAddr,u32)>> {
            let mut connections = Vec::new();
            connections.extend(Self::get_established_connections_from_proc("/proc/net/tcp", false)?);
            connections.extend(Self::get_established_connections_from_proc("/proc/net/tcp6", true)?);
            Ok(connections)
        }
    }
    
    #[derive(Hash, Eq, PartialEq, Debug, Deserialize,Serialize,Clone)]
    pub struct Process{
        pub pid: String,
        pub exe: String,
        pub cmd: String,
        pub cmdline: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Processes {
        pub processes: HashSet<Process>,
        pub new_processes: HashSet<Process>,
    }

    impl Processes {
        pub fn new() -> Self {
             
            Self {processes : Self::get_current_processes(),
            new_processes : Self::get_current_processes(),}
         
        }
        fn get_current_processes() -> HashSet<Process> {
            let mut processes = HashSet::new();
            
            // The /proc directory contains all running processes
            let proc_dir = "/proc";
            
            let entries = fs::read_dir(proc_dir).unwrap_or_else(|error| {
                if error.kind() == ErrorKind::NotFound {
                        panic!("{} directory not found, cannot get list of processes: {error:?}",proc_dir);
                    }
                else {
                    panic!("Shit: {error:?}");
                }
            }
            );
            
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                
                // Check if the entry is a directory and contains only digits (process id)
                if path.is_dir() {
                    if let Some(pid_str) = path.file_name().and_then(|s| s.to_str()) {
                        if pid_str.chars().all(char::is_numeric) {
                            
                            //Read the cmdline file to get the path
                            let cmdline_path = path.join("cmdline");
                            let mut file = fs::File::open(&cmdline_path).unwrap_or_else(|error| {
                                if error.kind() == ErrorKind::NotFound {
                                    panic!("{} file was not found {error:?}",cmdline_path.display());
                                }
                                else {
                                    panic!("I be broken {error:?}");
                                }
                            });
                            let mut cmdline = String::new();
                            file.read_to_string(&mut cmdline).unwrap_or_else(|error| {
                                match error.kind() {
                                    ErrorKind::NotFound =>  { panic!("{} file was not found {error:?}", cmdline)},
                                    ErrorKind::PermissionDenied =>  { panic!("{} permission denied {error:?}", cmdline)},
                                    _ => { panic!("Shit be hard dude {error:?}")},
                                }
                            });
                            let executable_path = cmdline.split('\0').next().unwrap_or("").split(" ").next().unwrap().to_string();

                            //Get the symlink path to the executable
                            let exe_path = path.join("exe");
                            let exe_file = fs::read_link(&exe_path).unwrap_or_else(|error| {
                                if error.kind() == ErrorKind::PermissionDenied {
                                    //println!("Permission denied");
                                    let nopath = PathBuf::new();
                                    nopath
                                }
                                else {
                                    //println!("I don't know {error:?} {exe_path:?}");
                                    let nopath = PathBuf::new();
                                    nopath
                                }
                            });
                            //Insert into a struct
                            processes.insert(Process {exe: exe_file.display().to_string(),pid: pid_str.to_string(),cmd: executable_path,cmdline: cmdline});
                        }
                    }
                }
            }
            
            
            processes
        }

        pub fn get_new_processes(&mut self) -> HashSet<Process> {
            let all_processes: HashSet<Process> = Self::get_current_processes();
            let new_processes: HashSet<Process> = all_processes.difference(&self.processes).cloned().collect::<HashSet<Process>>();
            //panic!("{} new processes", new_processes.len());
            self.processes = all_processes;
            new_processes
        }
    }


}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use gethostname::gethostname;
    extern crate paho_mqtt as mqtt;
    //use inventory_client::InventoryTransport;
    use env_logger::*;
    use pnet::ipnetwork::IpNetwork;
    use std::process::{Command, Stdio};

    #[test]
    fn get_system_info_which_succeeds(){
        let result = sys_interagator::SystemInfo::new();
        let mut this_machine_id = fs::read_to_string("/etc/machine-id").expect("/etc/machine-id not found or can't be opened");
        if this_machine_id.ends_with('\n'){
            this_machine_id.pop();
        }
        assert_eq!(result.agent_id,this_machine_id)
    }

    #[test]
    fn get_hostname_which_succeeds(){
        let result = sys_interagator::SystemInfo::new();
        let this_machine_hostname = gethostname().into_string().unwrap();
        assert_eq!(result.hostname,this_machine_hostname)
    }

    #[test]
    fn get_ipv4_addresses_which_succeed(){
        let result = sys_interagator::SystemInfo::get_ipaddresses("v4");
        assert!(result.iter().any(|n| IpNetwork::V4(n.parse().unwrap()).is_ipv4()));
    }


    #[test]
    fn get_ipv6_addresses_which_succeed(){
        let result = sys_interagator::SystemInfo::get_ipaddresses("v6");
        assert!(result.iter().any(|n| IpNetwork::V6(n.parse().unwrap()).is_ipv6()));
    }
    
    #[test]
    fn get_all_processes_which_succeed(){
       let result = sys_interagator::Processes::new();
        assert!(if result.processes.iter().count()>0{true}else{false});
    }

    #[test]
    fn get_new_processes_which_succeed(){
        let mut processes = sys_interagator::Processes::new();
        //spawn a process
        let _cmd_output = Command::new("ls")
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to execute process");
        let result = processes.get_new_processes();
        assert!(if result.iter().count()>0{true}else{false});
    }
}

