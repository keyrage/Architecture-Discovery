Agent schema

Descripton
Provides information about the agent (This may be the same as the node being interrogated or it may be a remote agent)

{
    agent_id :
    hostname :
    host : {
        bios_manufacturer :
        cpu_cores :
        memory_gb :
    }
    os_info : {
        os_family :
        os_version :
        os_edition :
    }
}

Node Schema

{
    node_id :
    node_type :
    hostname :
    ipv4_addresses : [12.12.1.2/24,1.1.1.1/8]
    ipv6_addresses : [xxx/xx,xxx/xx]
    ipv4_gateways : [xxxx,xxxx]
    ipv6_gateways : [xxxx:xxxx]
}

Process Schema

{
    node_id :
    node_snapshot_start_time :
    node_snapshot_stop_time :
    processes : [{
        process_name :
        process_path :
        process_listening_ports [TCP/25,UDP/21,TCP/3000,UDP/3001]
        process_connected_sessions [{
            source_port :
            destination_port :
            destination_ip :
        }]
        process_launch_time
    }]
}

DNS Records schema

{
    node_id :
    node_snapshot_start_time :
    node_snapshot_stop_time :
    dns_entries : [
        {
            dns_type :
            dns_name :
            dns_address : [12.12.12.12,1,1,1,1]
        }
    ]
}
