#/usr/bin/env bash

readonly DESTINATION=$1
readonly DEST_PATH="/home/djhunter67/app"

# Look in the target/release/ directory for the binary and that is the name of the application
APP_NAME=$(ls target/release/ | head -n 1)


if [ -z "$DESTINATION" ]; then
    echo "Destination is required"
    echo "EXAMPLE: ./deploy.sh 192.168.1.10"
    exit 1
fi

# mkdir the target directory on the target machine
# ssh -t $DESTINATION "mkdir -p ${DEST_PATH}" #2>&1 > /dev/null

# Build the Rust application
cargo build --release #--quiet #2>&1 > /dev/null

# Copy the binary to the server
rsync -Pauvht --stats {.env,static,target/release/$APP_NAME} $DESTINATION:${DEST_PATH} 2>&1 > /dev/null

# Check if the last command was successful
if [ $? -eq 0 ]; then
    # Echo in green
    echo -e "\e[32m\n\n############################################################\e[0m"
    echo -e "\e[32m################Deployment successful###################\e[0m"
    echo -e "\e[32m############################################################\e[0m"
else
    # Echo in red
    echo -e "\e[31m\n\n############################################################\e[0m"
    echo -e "\e[31m################Deployment failed###################\e[0m"
    echo -e "\e[31m############################################################\e[0m"
    exit 1
fi

# Check if the destination server has a systemctl service for the $APP_NAME
ssh -t $DESTINATION "systemctl status ${APP_NAME} 2>&1 > /dev/null" 
# Check if the last command was successful
if [ $? -eq 0 ]; then
    # Echo in green
    echo -e "\e[32m\n\n############################################################\e[0m"
    echo -e "\e[32m################Service is running###################\e[0m"
    echo -e "\e[32m############################################################\e[0m"
else
    # Echo in red
    echo -e "\e[31m\n\n############################################################\e[0m"
    echo -e "\e[31m################Service is not running###################\e[0m"
    echo -e "\e[31m############################################################\e[0m"

    # Create the service file
    cat > $APP_NAME.service <<EOF
[Unit]
Description=aj_studying
After=network.target

[Service]
Type=simple
User=djhunter67
WorkingDirectory=/home/djhunter67/app
ExecStart=/home/djhunter67/app/aj_studying
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

    SERVICE_FILE="$DEST_PATH/$APP_NAME.service"

    # Copy the service file to the server
    rsync -Pauvht --stats $APP_NAME.service $DESTINATION:~/ 2>&1 > /dev/null
    echo -e "\e[33mCopying the service file: ${APP_NAME}.service @ ${DESTINATION}\e[0m"
    ssh -t $DESTINATION "sudo cp ~/$APP_NAME.service /etc/systemd/system/" 
    # Check if the last command was successful
    if [ $? -eq 0 ]; then
	# Echo in green
	echo -e "\e[32m\n\n############################################################\e[0m"
	echo -e "\e[32m################Service file copied successfully###################\e[0m"
	echo -e "\e[32m############################################################\e[0m"
    else
	# Echo in red
	echo -e "\e[31m\n\n############################################################\e[0m"
	echo -e "\e[31m################Service file copy failed###################\e[0m"
	echo -e "\e[31m############################################################\e[0m"
	exit 1
    fi

    # Reload the systemctl daemon
    echo -e "\e[33mReloading the systemd daemon:  @ ${DESTINATION}\e[0m"
    # ignore all of the output
    ssh -t $DESTINATION "sudo systemctl daemon-reload" 

    # Check if the last command was successful
    if [ $? -eq 0 ]; then
	# Echo in green
	echo -e "\e[32m\n\n############################################################\e[0m"
	echo -e "\e[32m################Daemon reloaded successfully###################\e[0m"
	echo -e "\e[32m############################################################\e[0m"
	
    else
	# Echo in red
	echo -e "\e[31m\n\n############################################################\e[0m"
	echo -e "\e[31m################Daemon reload failed###################\e[0m"
	echo -e "\e[31m############################################################\e[0m"
    fi

    # Start the service
    echo -e "\e[33mStarting the service: ${APP_NAME} @ ${DESTINATION}\e[0m"
    ssh -t $DESTINATION "sudo systemctl start $APP_NAME"

    # Check if the last command was successful
    if [ $? -eq 0 ]; then
	# Echo in green
	echo -e "\e[32m\n\n############################################################\e[0m"
	echo -e "\e[32m################Service started successfully###################\e[0m"
	echo -e "\e[32m############################################################\e[0m"
	exit 0
    else
	# Echo in red
	echo -e "\e[31m\n\n############################################################\e[0m"
	echo -e "\e[31m################Service start failed###################\e[0m"
	echo -e "\e[31m############################################################\e[0m"
	exit 1
    fi
fi

# Restart the service
echo -e "\e[33mRestarting the service: ${APP_NAME} @ ${DESTINATION}\e[0m"
ssh -t $DESTINATION "sudo systemctl restart $APP_NAME" 

# Check if the last command was successful
if [ $? -eq 0 ]; then
    # Echo in green
    echo -e "\e[32m\n\n############################################################\e[0m"
    echo -e "\e[32m################Service restarted successfully###################\e[0m"
    echo -e "\e[32m############################################################\e[0m"
else
    # Echo in red
    echo -e "\e[31m\n\n############################################################\e[0m"
    echo -e "\e[31m################Service restart failed###################\e[0m"
    echo -e "\e[31m############################################################\e[0m"
    exit 1
fi
