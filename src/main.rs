use ctrlc;
use std::process;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use paho_mqtt as mqtt;

const BROKER: &str = "tcp://192.168.1.127:1886";
const CLIENT: &str = "Pi";
const BASE_TOPIC: &str = "rasp/";

fn main() {
    ctrlc::set_handler(move || {
        println!("Unclean shutdown!");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    // Initialize the client
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(BROKER)
        .client_id(CLIENT)
        .finalize();

    let cli = mqtt::Client::new(create_opts).unwrap();

    let lwt = mqtt::MessageBuilder::new()
        .topic("rasp/status")
        .payload("0")
        .retained(true)
        .finalize();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    println!("Connecting to the MQTT server...");

    if let Err(e) = cli.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    cli.publish(mqtt::Message::new("rasp/status", "1", 1))
        .unwrap();

    let rx = cli.start_consuming();
    cli.subscribe_many(&["rasp/button", "rasp/door_state"], &[2, 2])
        .unwrap();

    // Init state
    // This entire part needs to be rethought
    let mut garage_door_some: Option<GarageDoor> = None;

    let mut reconnection_attempts: u8 = 0;
    while garage_door_some.is_none() {
        for msg in rx.iter() {
            if let Some(m) = msg {
                if m.topic() == "rasp/door_state" {
                    let state: DoorState;
                    match DoorState::from_str(&m.payload_str()) {
                        Ok(m) => {
                            state = m;
                        }
                        Err(_) => {
                            eprintln!("Invalid state in topic 'door_state'!");
                            state = DoorState::Unknown;
                        }
                    }
                    garage_door_some = Some(GarageDoor::new(state));
                    break;
                }
            } else {
                try_reconnect(&cli);
            }
        }
    }

    let mut time_of_button_press = SystemTime::now();
    let mut garage_door = garage_door_some.unwrap();

    println!(
        "Finished initializing. Garage door state: {:?}",
        garage_door.get_state()
    );

    // Main loop
    for msg in rx.iter() {
        if let Some(m) = msg {
            println!(
                "Received message '{}' with QoS '{}'",
                m.payload_str(),
                m.qos()
            );

            if m.payload_str() == "1" {
                if let Ok(time_since) = time_of_button_press.elapsed() {
                    if time_since.as_secs() > 1 {
                        garage_door.trigger();
                        time_of_button_press = SystemTime::now();
                    }

                } else {
                    garage_door.trigger();
                    time_of_button_press = SystemTime::now();
                }
            } else if m.payload_str() == "0" {
                continue;
            } else {
                println!(
                    "Unknown payload {}, on topic {}",
                    m.payload_str(),
                    m.topic()
                );
            }

            match cli.publish(mqtt::Message::new("rasp/button", "0", 2)) {
                Err(_) => eprintln!("Error reseting button to zero state."),
                Ok(_) => (),
            }
        } else {
            try_reconnect(&cli);
        }
    }

    disconnect_now(&cli);
    println!("Exiting");
}

fn disconnect_now(cli: &mqtt::Client) {
    if cli.is_connected() {
        println!("Disconnecting from the MQTT server...");
        cli.publish(mqtt::Message::new("rasp/status", "0", 1))
            .unwrap();
        cli.disconnect(None).unwrap();
    } else {
        println!("Already disconnected");
    }
}

fn try_reconnect(cli: &mqtt::Client) {
    if !cli.is_connected() {
        println!("Reconnecting to the MQTT server...");
        cli.reconnect().unwrap();
    } else {
        println!("Already connected");
    }
}

struct GarageDoor {
    state: DoorState,
}

impl GarageDoor {
    fn new(current_state: DoorState) -> GarageDoor {
        GarageDoor {
            state: current_state,
        }
    }

    fn get_state(&self) -> &DoorState {
        return &self.state;
    }

    fn trigger(&mut self) {
        println!("Trigger door");
        return;
        match self.state {
            DoorState::Open => todo!(),
            DoorState::Closed => todo!(),
            DoorState::Moving => todo!(),
            DoorState::Unknown => todo!(),
        }
    }
}

#[derive(Debug)]
enum DoorState {
    Open,
    Closed,
    Moving,
    Unknown,
}

impl FromStr for DoorState {
    type Err = ();
    fn from_str(input: &str) -> Result<DoorState, Self::Err> {
        match input {
            "Open" => Ok(DoorState::Open),
            "Closed" => Ok(DoorState::Closed),
            "Moving" => Ok(DoorState::Moving),
            "Unknown" => Ok(DoorState::Unknown),
            _ => Err(()),
        }
    }
}
