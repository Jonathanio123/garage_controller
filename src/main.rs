use ctrlc;
use mqtt::Client;
#[cfg(target_arch = "arm")]
use rppal::gpio::Gpio;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use paho_mqtt as mqtt;

const BROKER: &str = "tcp://192.168.1.127:1886";

// Release constants
#[cfg(debug_assertions)]
const CLIENT: &str = "Pi_debug";
#[cfg(debug_assertions)]
const TOPIC_STATUS: &str = "rasp_debug/status";
#[cfg(debug_assertions)]
const TOPIC_BUTTON: &str = "rasp_debug/button";

// Debug constants
#[cfg(not(debug_assertions))]
const CLIENT: &str = "Pi";
#[cfg(not(debug_assertions))]
const TOPIC_STATUS: &str = "rasp/status";
#[cfg(not(debug_assertions))]
const TOPIC_BUTTON: &str = "rasp/button";
#[cfg(target_arch = "arm")]
const RELAY5: u8 = 23;
static STOPPING_NOW: AtomicBool = AtomicBool::new(false);

fn main() {
    #[cfg(debug_assertions)]
    env_logger::init();
    #[cfg(target_arch = "arm")]
    println!(
        "Running armv7 build, version: {}",
        env!("CARGO_PKG_VERSION")
    );
    #[cfg(target_arch = "arm")]
    let mut pin = Gpio::new().unwrap().get(RELAY5).unwrap().into_output();
    #[cfg(target_arch = "arm")]
    pin.set_high();

    let cli = init_client_connection();
    let cli_clone = cli.clone(); // To be used in SIGINT handler

    // SIGINT handler
    ctrlc::set_handler(move || {
        STOPPING_NOW.store(true, Ordering::Relaxed);
        println!("\nSIGINT received, disconnecting...");
        let offline_msg = mqtt::MessageBuilder::new()
            .topic(TOPIC_STATUS)
            .payload("0")
            .qos(1)
            .retained(true)
            .finalize();

        cli_clone.publish(offline_msg).ok();
        cli_clone
            .disconnect_after(Duration::from_millis(1000))
            .expect("Error when disconnecting.\nUnclean shutdown...");
        println!("Disconnected, exiting...");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let mut time_of_button_press = SystemTime::now();

    // Main loop
    println!("Finished initializing.");
    loop {
        // Park thread when SIGINT is received and
        // wait for SIGINT handler to exit the process.
        if STOPPING_NOW.load(Ordering::Relaxed) {
            thread::park();
            continue;
        }

        let rx = cli.start_consuming();
        cli.subscribe(TOPIC_BUTTON, 2).unwrap();

        for msg in rx.iter() {
            if let Some(msg) = msg {
                #[cfg(debug_assertions)]
                println!(
                    "Received message '{}' with QoS '{}'",
                    msg.payload_str(),
                    msg.qos()
                );

                // Topic button BEGIN
                if msg.topic() == TOPIC_BUTTON {
                    if msg.payload_str() == "1" {
                        match time_of_button_press.elapsed() {
                            Ok(time_since) => {
                                // Prevent accidental double clicks
                                if time_since.as_secs() > 1 {
                                    println!("Button pressed");
                                    #[cfg(target_arch = "arm")]
                                    {
                                        pin.set_low();
                                        thread::sleep(Duration::from_millis(100)); // Needed for the relay to react.
                                        pin.set_high();
                                    }
                                    time_of_button_press = SystemTime::now();
                                }
                            }
                            Err(e) => {
                                eprintln!("System time error, elapsed time: {:?}", e);
                                time_of_button_press = SystemTime::now();
                            }
                        }

                        // This may not be necessary but is nice when debugging
                        // #[cfg(debug_assertions)] ?
                        match cli.publish(mqtt::Message::new(TOPIC_BUTTON, "0", 2)) {
                            Err(e) => eprintln!("Error resetting button to zero state.\n\t{}", e),
                            Ok(_) => (),
                        }
                    }
                // Topic button END
                } else {
                    // Unknown messages are printed
                    println!(
                        "Unknown payload {}, on topic {}",
                        msg.payload_str(),
                        msg.topic()
                    );
                }
            } else {
                try_reconnect(&cli);
                break;
            }
        }
    }
}

fn try_reconnect(cli: &mqtt::Client) {
    println!("Lost connection.\nReconnecting to the MQTT server...");
    let mut attempts: u8 = 0;
    while !cli.is_connected() {
        // Do not try to reconnect if SIGINT has been received.
        if STOPPING_NOW.load(Ordering::Relaxed) {
            return;
        }
        attempts += 1;
        match cli.reconnect() {
            Ok(_) => break,
            Err(e) => {
                if attempts >= 120 {
                    eprintln!("Unable to reconnect giving up!\nExiting...");
                    process::exit(2)
                }
                eprintln!(
                    "Unable to reconnect to MQTT server, sleeping for 10 seconds...\n\t{}\n\tAttempt: {}",
                    e.to_string(), attempts.to_string()
                );
                thread::sleep(Duration::from_secs(10));
            }
        }
    }
    // Publish "1" to status topic
    cli.publish(is_online_msg_builder()).unwrap();
    println!("Successfully reconnected to MQTT server.")
}

fn is_online_msg_builder() -> mqtt::Message {
    return mqtt::MessageBuilder::new()
        .topic(TOPIC_STATUS)
        .payload("1")
        .qos(1)
        .retained(true)
        .finalize();
}

fn init_client_connection() -> Client {
    // Initialize the client
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(BROKER)
        .client_id(CLIENT)
        .finalize();

    let cli = mqtt::Client::new(create_opts).unwrap();

    let lwt = mqtt::MessageBuilder::new()
        .topic(TOPIC_STATUS)
        .payload("0")
        .qos(1)
        .retained(true)
        .finalize();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(10))
        .clean_session(true)
        .will_message(lwt)
        .finalize();

    println!("Connecting to the MQTT server...");

    if let Err(e) = cli.connect(conn_opts) {
        println!("Unable to connect:\n\t{:?}", e.to_string());
        process::exit(1);
    }

    println!("Connected! MQTT server: {}", BROKER);

    // Publish "1" to status topic
    cli.publish(is_online_msg_builder()).unwrap();

    // Reset button to 0, may not be needed
    cli.publish(mqtt::Message::new(TOPIC_BUTTON, "0", 1))
        .unwrap();

    return cli;
}
