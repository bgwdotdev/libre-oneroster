#!/bin/sh
# This script does the following:
#   * Changes the ownership to /etc/opt/oneroster to the low-priv `oneroster` account
#   * Changes the ownership to /var/opt/oneroster to the low-priv `oneroster` account
#   * Switches the container to run as `oneroster`
#   * Launches the binary

set -e

OR_CONFIG="/etc/opt/oneroster"
OR_DATA="/var/opt/oneroster"

if [ "$1" = 'oneroster' ]; then
    if [ $(stat -c %u $OR_CONFIG) != $(id -u oneroster) ]; then
        chown --recursive oneroster:oneroster $OR_CONFIG || echo "Failed to chown $OR_CONFIG"
    fi

    if [ $(stat -c %u $OR_DATA) != $(id -u oneroster) ]; then
        chown --recursive oneroster:oneroster $OR_DATA || echo "Failed to chown $OR_DATA"
    fi
fi

exec gosu oneroster $@ 
