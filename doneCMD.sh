#!/bin/bash

# Available Variables
# - ${CLD_DIR}
# - ${CLD_PATH}
# - ${CLD_HASH}
# - ${CLD_TYPE}
# - ${CLD_RESTAPI}
# - ${CLD_SIZE}
# - ${CLD_STARTTS}
LOCALPATH="${CLD_DIR}/${CLD_PATH}"
NOWTS=$(date +%s)
RESTAPI="localhost:3000"
JOBPATH="/home/pi/TheUploader/jobs"

# skip tasks finished too soon, more likely the program just restarted
if [[ $(($NOWTS - $CLD_STARTTS)) -le 10 ]];then
        echo "STARTTS less then 10s, should ignore this task"
        exit 0
fi

# this is called when the whole task is finished
if [[ ${CLD_TYPE} == "torrent" ]]; then
    # to stop the task
    /usr/bin/curl --data "stop:${CLD_HASH}" "http://${RESTAPI}/api/torrent"

    echo "Removing Completed: ${LOCALPATH}"
    # remove the folder since all files to be uploaded has been copied to jobs folder
    rm -rf "${LOCALPATH}"

    # to remove the task
    /usr/bin/curl --data "delete:${CLD_HASH}" "http://${RESTAPI}/api/torrent"
fi

# this is called when one of the files is finish, here skips files with size smaller than 5MB
if [[ ${CLD_TYPE} == "file" ]] && [[ ${CLD_SIZE} -gt $((5*1024*1024)) ]]; then
    mv "${LOCALPATH}" "${JOBPATH}"
fi