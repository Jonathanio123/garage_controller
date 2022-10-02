#!/bin/bash

if ls /root &> /dev/null; then
    echo "Could open root folder. Proceeding..."
    
else
    echo "Could not open root folder, try running as root."
    exit 1
fi


if ls /root/garage_controller &> /dev/null; then
    echo "Install found"
    read -p "Do you want to reinstall? This will remove the contents of /root/garage_controller! [y/N]" -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]
    then
        echo Stopping service and removing old files
        systemctl stop garage_controller
        systemctl disable garage_controller
        rm -r /root/garage_controller
        rm /etc/systemd/system/garage_controller.service
    else
        read -p "Do you want to Uninstall? This will remove the contents of /root/garage_controller! [y/N]" -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]
        then
            echo Stopping service and removing old files
            systemctl stop garage_controller
            systemctl disable garage_controller
            rm -r /root/garage_controller
            rm /etc/systemd/system/garage_controller.service
            exit 0
            
        else
            exit 0
        fi
    fi
else
    echo "Install not found"
fi

echo "Installing daemon"

mkdir /root/garage_controller
if ls ./garage_controller &> /dev/null; then
    echo "Binary found in current folder"
    cp ./garage_controller /root/garage_controller
    
    elif ls ./target/armv7-unknown-linux-gnueabihf/release/garage_controller &> /dev/null; then
    echo "Binary found under target/release"
    cp ./target/armv7-unknown-linux-gnueabihf/release/garage_controller /root/garage_controller
else
    rm -r /root/garage_controller
    echo "Binary not found!"
    echo "Buld app with 'cargo build --target=armv7-unknown-linux-gnueabihf --release'"
    exit 2
fi

chmod u=rx /root/garage_controller/garage_controller

cp ./garage_controller.service /etc/systemd/system
echo "Remember to enable and start garage_controller.service"


#cp ./target/armv7-unknown-linux-gnueabihf/release/garage_controller /root/garage_controller