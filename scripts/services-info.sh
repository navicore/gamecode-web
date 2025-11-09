#!/bin/bash

# Simple service information script for macOS
# Shows all services, their locations, and status

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}macOS Services Overview${NC}"
echo "================================"
echo ""

# Get all services from launchctl
echo -e "${GREEN}Currently Loaded Services:${NC}"
echo ""

# Header
printf "%-40s %-10s %-8s %s\n" "SERVICE" "STATUS" "PID" "LOCATION"
echo "--------------------------------------------------------------------------------"

# Find all plist files
for dir in ~/Library/LaunchAgents /Library/LaunchAgents /Library/LaunchDaemons; do
    if [[ -d "$dir" ]]; then
        for plist in "$dir"/*.plist; do
            if [[ -f "$plist" ]]; then
                service_name=$(basename "$plist" .plist)
                
                # Skip Apple services unless verbose
                if [[ "$service_name" == com.apple.* ]] && [[ "${1:-}" != "-v" ]]; then
                    continue
                fi
                
                # Get status from launchctl
                launchctl_info=$(launchctl list 2>/dev/null | grep "^[0-9-]*[[:space:]]*[0-9-]*[[:space:]]*$service_name$" || echo "")
                
                if [[ -n "$launchctl_info" ]]; then
                    pid=$(echo "$launchctl_info" | awk '{print $1}')
                    exit_code=$(echo "$launchctl_info" | awk '{print $2}')
                    
                    if [[ "$pid" != "-" ]]; then
                        status="${GREEN}Running${NC}"
                        pid_display="$pid"
                    else
                        status="${YELLOW}Stopped${NC}"
                        pid_display="-"
                    fi
                else
                    # Check if it's a system daemon
                    if [[ "$dir" == "/Library/LaunchDaemons" ]]; then
                        system_status=$(launchctl print system/"$service_name" 2>/dev/null | grep "state = " || echo "")
                        if [[ "$system_status" == *"running"* ]]; then
                            # Try to find PID from ps
                            pid=$(ps aux | grep "$service_name" | grep -v grep | awk '{print $2}' | head -1)
                            status="${GREEN}Running${NC}"
                            pid_display="${pid:-system}"
                        elif [[ -n "$system_status" ]]; then
                            status="${YELLOW}Stopped${NC}"
                            pid_display="-"
                        else
                            status="${RED}NotLoaded${NC}"
                            pid_display="-"
                        fi
                    else
                        status="${RED}NotLoaded${NC}"
                        pid_display="-"
                    fi
                fi
                
                # Shorten path for display
                location="${dir/#$HOME/~}"
                
                printf "%-40s ${status}    %-8s %s\n" "$service_name" "$pid_display" "$location"
            fi
        done
    fi
done

echo ""
echo -e "${CYAN}Quick Actions:${NC}"
echo "  Start service:   launchctl start <service-name>"
echo "  Stop service:    launchctl stop <service-name>"
echo "  Load service:    launchctl load ~/Library/LaunchAgents/<service>.plist"
echo "  Unload service:  launchctl unload ~/Library/LaunchAgents/<service>.plist"
echo ""
echo -e "${CYAN}View all services (including Apple):${NC} $0 -v"