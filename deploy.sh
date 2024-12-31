#/usr/bin/env bash

readonly DESTINATION=$1
readonly DEST_PATH="/home/ripley/app"
readonly TARGET_ARCH=aarch64-unknown-linux-gnu
readonly SOURCE_PATH=target/${TARGET_ARCH}/release/kid_data


# Look in the target/release/ directory for the binary and that is the name of the application
APP_NAME=$(ls target/$TARGET_ARCH/release/ | head -n 1)

echo "APP_NAME: $APP_NAME"

if [ -z "$DESTINATION" ]; then
    echo "Destination is required"
    echo "EXAMPLE: ./deploy.sh 192.168.1.10"
    exit 1
fi

# mkdir the target directory on the target machine
# ssh -t $DESTINATION "mkdir -p ${DEST_PATH}" #2>&1 > /dev/null

# Build the Rust application
echo -e "\e[33mBuilding the Rust application for Release\e[0m"
# cargo build --release --quiet # 2>&1 > /dev/null
cross build --release --target=aarch64-unknown-linux-gnu

echo -e "\e[33mCopying the build files to the server\e[0m"
rsync -Pauvht --stats {settings,.env,static,$SOURCE_PATH} $DESTINATION:${DEST_PATH} --exclude target --exclude .git --exclude .github --exclude .gitignore --exclude aj_quiz.log --exclude scan_yam.log --exclude README.md --exclude deploy.sh --exclude target  --exclude tests --exclude .cargo --exclude \*\~ 2>&1 > /dev/null
# rsync -Paurvht --stats ./ $DESTINATION:${DEST_PATH} --exclude target --exclude .git --exclude .github --exclude .gitignore --exclude aj_quiz.log --exclude scan_yam.log --exclude README.md --exclude deploy.sh --exclude target  --exclude tests --exclude .cargo --exclude \*\~ 

# Check if the last command was successful
if [ $? -eq 0 ]; then
    # Echo in green
    echo -e "\e[32m\n\n############################################################\e[0m"
    echo -e "\e[32m################Transfer Successful###################\e[0m"
    echo -e "\e[32m############################################################\e[0m\n"
else
    # Echo in red
    echo -e "\e[31m\n\n############################################################\e[0m"
    echo -e "\e[31m################Transfer Failed###################\e[0m"
    echo -e "\e[31m############################################################\e[0m"
    exit 1
fi

# echo -e "\n\e[33mBuilding the Rust application on the server\e[0m"
# ssh -t $DESTINATION "~/.cargo/bin/cargo build --manifest-path ${DEST_PATH}/Cargo.toml --release 2>&1 > /dev/null"

# # Check if the last command was successful
# if [ $? -eq 0 ]; then
#     # Echo in green
#     echo -e "\e[32m\n\n############################################################\e[0m"
#     echo -e "\e[32m################Build Successful###################\e[0m"
#     echo -e "\e[32m############################################################\e[0m\n"
# else
#     # Echo in red
#     echo -e "\e[31m\n\n############################################################\e[0m"
#     echo -e "\e[31m################Build Failed###################\e[0m"
#     echo -e "\e[31m############################################################\e[0m"
#     exit 1
# fi


# Check if the destination server has a systemctl service for the $APP_NAME
ssh -t $DESTINATION "systemctl status ${APP_NAME} 2>&1 > /dev/null" 
# Check if the last command was successful
if [ $? -eq 0 ]; then
    # Echo in green
    echo -e "\e[32m\n\n############################################################\e[0m"
    echo -e "\e[32m################Service is Running###################\e[0m"
    echo -e "\e[32m############################################################\e[0m"
else
    # Echo in red
    echo -e "\e[31m\n\n############################################################\e[0m"
    echo -e "\e[31m################Service is Not Running###################\e[0m"
    echo -e "\e[31m############################################################\e[0m"

    # Create the service file
    cat > $APP_NAME.service <<EOF
[Unit]
Description=kid_data
After=network.target

[Service]
Type=simple
User=ripley
WorkingDirectory=/home/ripley/app
ExecStart=/home/ripley/app/kid_data
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

    SERVICE_FILE="$DEST_PATH/kid_data.service"

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
    exit 0
else
    # Echo in red
    echo -e "\e[31m\n\n############################################################\e[0m"
    echo -e "\e[31m################Service restart failed###################\e[0m"
    echo -e "\e[31m############################################################\e[0m"
    exit 1
fi



# set -o errexit
# set -o nounset
# set -o pipefail
# set -o xtrace

# USER_NAME="ripley"
# IP=192.168.110.51

# readonly TARGET_HOST=$USER_NAME@$IP
# readonly TARGET_PATH=/home/$USER_NAME
# readonly TARGET_ARCH=aarch64-unknown-linux-gnu
# readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/kid_data

# cross build --release --target=${TARGET_ARCH}
# rsync -Pauvht --stats ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
# ssh -t ${TARGET_HOST} sudo systemctl restart antenna_switcher_api.service
