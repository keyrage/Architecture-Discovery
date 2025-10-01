pub mod inventory_client {
    use std::time::Duration;
    use std::{thread, time};
    use log::*;
    use std::error::Error;
    use std::collections::VecDeque;
    use uuid::Uuid;
    use serde::{Deserialize, Serialize};
    extern crate paho_mqtt as mqtt;
    
    pub struct InventoryTransport {
        pub connected: bool,
        pub url: String,
        client: mqtt::Client,
        pub retry_delay_secs: u64,
        pub max_retries:u8,
        message_queue: VecDeque<Message>,
        pub queue_length: usize,
        pub max_queue_length: usize,
    }

    struct Message {
        topic: String,
        payload: String,
        qos: u8,
    }

    impl InventoryTransport {
        pub fn new(server: String,port: u16,clientid: String) -> Self {
            let server_uri = format!("tcp://{}:{}",server,port.to_string());
            debug!("MQTT Server URI: {}", server_uri);
            let mqtt_options = mqtt::CreateOptionsBuilder::new()
                .server_uri(server_uri.to_owned())
                .client_id(&clientid)
                .finalize();
            Self {
                connected: false,
                url: server_uri,
                client: mqtt::Client::new(mqtt_options).unwrap(),
                retry_delay_secs: 1,
                max_retries: 5,
                message_queue: VecDeque::new(),
                queue_length: 0,
                max_queue_length: 100,
            }

        }
        pub fn connect(&mut self) -> Result<(),mqtt::Error>{
            let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .retry_interval(Duration::from_secs(self.retry_delay_secs))
            .connect_timeout(Duration::from_secs(20))
            .automatic_reconnect(Duration::from_secs(5),Duration::from_secs(3600))
            .finalize();

            //let max_retries = 10;
            let wait = time::Duration::from_secs(self.retry_delay_secs);
            //let mut server_conn_response = mqtt::ServerResponse::new(); // TODO: Remove
            
            for _ in 0..self.max_retries {
                if self.client.is_connected() == true {
                    debug!{"Is connected? {}",self.client.is_connected()};
                    self.connected = true;
                    info!{"Connected to MQTT"};
                    return Ok(())
                }
                else
                {
                    self.client.connect(conn_opts.clone()).unwrap_or_else(|error| {
                        match error {
                            ref TcpTlsConnectFailure => {warn!("Failed to connect to {}",self.url); mqtt::ServerResponse::new()}
                            _ => {warn!("Unknown error, failed to connect to {}: {error:?}",self.url); mqtt::ServerResponse::new()}
                        }
                    }); //Todo, the ? basically returns the mqtt Error back to the calling code in Main, nothing else after this code is called..
                    //debug!{"Server Response is {}", server_conn_response.reason_code()};
                    //debug!{"serverui is {}",self.url};
                    //warn!{"Not yet connected to MQTT server: {}",self.url};
                    thread::sleep(wait);
                }

            }
            error!{"Failed to connect to {}",self.url};
            return Err(mqtt::Error::from("Failed to connect"))
        }
        pub fn disconnect(&mut self) -> Result<(),mqtt::Error>{
            if self.client.is_connected() == false {
                let _ = self.client.disconnect(None)?;
                let _ = self.connected == true;
                info!{"Disconnected from MQTT server"};
                Ok(())
            }
            else
            {
                info!{"Disconnected from MQTT server"};
                Ok(())
            }

        }
        pub fn send_message(&mut self, topic: &String, message: &String) -> Result<(),mqtt::Error>{
            debug!("Sending MQTT message {} on topic {}",message,topic);
            let payload: Vec<u8> = message.clone().into_bytes().to_vec();
            let mqtt_message = mqtt::Message::new(topic,payload,0);
            self.client.publish(mqtt_message)?;
            Ok(())
        }
        pub fn queue_message(&mut self, message: String, topic: String, qos: u8) -> Result<u64, Box<dyn Error>> {
            let new_message = Message {
                payload: message,
                topic: topic,
                qos: qos,
            };

            if self.queue_length < self.max_queue_length {
                self.message_queue.push_back(new_message);
                self.queue_length = self.message_queue.len();
                //debug!("Message: '{}' queued. Queue Size: {}", new_message.payload,self.queue_length);
                return Ok(self.queue_length.try_into().unwrap())
            }
            else
            {
                error!("Queue is full, dumping message.");
                debug!("Queue Size: {}, Max Queue Size: {}",self.queue_length,self.max_queue_length);
                return Err("Queue Full".into())
            }
            
        }
        pub fn flush_queue(&mut self){
            self.message_queue.clear();
            self.queue_length = self.message_queue.len();
        }
        pub fn process_message_queue(&mut self) -> Result<u64, Box<dyn Error>> {
            let message = self.message_queue.pop_front().unwrap();
            //self.send_message(&message.topic,&message.payload).unwrap_or_else(self.message_queue.pop_back(message) );
            Ok(1)
        }
    }
    
    #[derive(Serialize, Deserialize)]
    pub struct AgentInfo {
        pub agent_id: String,
        pub site_code: String,
        pub correlation_id: Uuid
    }

    impl AgentInfo {
        pub fn new(agent_id: String, site_code: String) -> Self{
            Self {
                agent_id: agent_id,
                site_code: site_code,
                correlation_id: Uuid::new_v4()
            }
        }
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    extern crate paho_mqtt as mqtt;
    use inventory_client::InventoryTransport;
    use env_logger::*;
    use serde_json::json;

    //Tests for InventoryTransport object
    #[test]
    fn connect_to_server_succeeds(){
        env_logger::init(); //Use RUST_LOG=debug cargo test -- --nocapture to see logs
        //needs a dummy server runnings to succeed
        let mut my_server = InventoryTransport::new("localhost".to_string(),9001,"dummy1".to_string());
        let result = my_server.connect();
        let _ = my_server.disconnect();
        assert!(result.is_ok());
    }
    
    #[test]
    fn connect_to_server_fails(){
        //needs a dummy server runnings to succeed
        let mut my_server = InventoryTransport::new("localhost".to_string(),9901,"fail1".to_string());
        let result = my_server.connect();
        let _ = my_server.disconnect();
        assert!(result.is_err());
    }

    #[test]
    fn send_message_succeeds(){
        //needs a dummy server to succeed
        let mut my_server = InventoryTransport::new("localhost".to_string(),9001,"dummy2".to_string());
        let _connection = my_server.connect().unwrap();
        let result = my_server.send_message(&"dummy".to_string(),&"message{test:this is a test}".to_string());
        let _ = my_server.disconnect();
        assert!(result.is_ok());
        
    }
    #[test]
    fn disconnect_succeeds(){
        let mut my_server = InventoryTransport::new("localhost".to_string(),9001,"dummy3".to_string());
        let _ = my_server.connect();
        let result = my_server.disconnect();
        assert!(result.is_ok());
    }
    #[test]
    fn queue_message_succeeds(){
        let mut my_server = InventoryTransport::new("localhost".to_string(),9001,"dummy1".to_string());
        let message = "A simple message".to_string();
        let topic = "testtopic".to_string();
        let qos = 0;
        let result = my_server.queue_message(message,topic,qos).unwrap();
        assert_eq!(result,1);
    }
    #[test]
    fn queue_many_messages_succeeds(){
        let mut my_server = InventoryTransport::new("localhost".to_string(),9001,"dummy1".to_string());
        for n in 1..101 {
            let message = "A simple message".to_string();
            let topic = "testtopic".to_string();
            let qos = 0;
            let result = my_server.queue_message(message,topic,qos).unwrap();
        }
        assert_eq!(my_server.queue_length,100);
    }
    #[test]
    fn queue_too_many_messages_succeeds(){
        let mut my_server = InventoryTransport::new("localhost".to_string(),9001,"dummy1".to_string());
        for n in 1..101 {
            let topic = "testtopic".to_string();
            let qos = 0;
            let message = "A simple message".to_string();
            let result = my_server.queue_message(message,topic,qos).unwrap();
        }
        let message = "A simple message".to_string();
        let topic = "testtopic".to_string();
        let qos = 0;
        let result = my_server.queue_message(message,topic,qos);
        assert!(result.is_err());
    }
    #[test]
    fn queue_json_message_succeeds(){
        let mut my_server = InventoryTransport::new("localhost".to_string(),9001,"dummy1".to_string());
        let topic = "testtopic".to_string();
        let qos = 0;
        let system_json = json!({
            "correlation_id":"sdfdfg343453456774564g!!$$$$FF@@",
            "agent_id":"123456567788990"
        });
        let system_json_string = serde_json::to_string(&system_json).unwrap();
        let message = system_json_string;
        let result = my_server.queue_message(message,topic,qos).unwrap();
        assert_eq!(result,1);
    }
    #[test]
    fn flush_queue(){
        let mut my_server = InventoryTransport::new("localhost".to_string(),9001,"dummy1".to_string());
        for n in 1..101 {
            let topic = "testtopic".to_string();
            let qos = 0;
            let message = "A simple message".to_string();
            let result = my_server.queue_message(message,topic,qos).unwrap();
        }
        my_server.flush_queue();
        assert_eq!(my_server.queue_length,0);
    }
    #[test]
    fn process_queue_succeeds() {
        let mut my_server = InventoryTransport::new("localhost".to_string(),9001,"dummy1".to_string());
        let topic = "testtopic".to_string();
        let qos = 0;
        let message = "A simple message".to_string();
        my_server.queue_message(message,topic,qos).unwrap();
        let result = my_server.process_message_queue().unwrap();
        assert_eq!(result,1);
    }

}