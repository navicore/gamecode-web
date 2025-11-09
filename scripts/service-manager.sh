#!/bin/bash

# macOS Service Manager - A k8s-like interface for launchd services
# Usage: ./service-manager.sh [command] [options]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Service discovery patterns
SERVICE_PATTERNS=(
    "com.gamecode.*"
    "com.ollama.*"
    "com.cloudflare.*"
    "homebrew.mxcl.*"
    "dev.*"
    "local.*"
)

# Common service paths
SERVICE_PATHS=(
    "$HOME/Library/LaunchAgents"
    "/Library/LaunchAgents"
    "/Library/LaunchDaemons"
    "/System/Library/LaunchAgents"
    "/System/Library/LaunchDaemons"
)

function print_header() {
    echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${CYAN}  macOS Service Manager - LaunchD Control Interface${NC}"
    echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════════════════${NC}"
}

function get_service_status() {
    local service=$1
    local status=$(launchctl list | grep "$service" 2>/dev/null || echo "")
    
    if [[ -z "$status" ]]; then
        echo "not-loaded"
    else
        local pid=$(echo "$status" | awk '{print $1}')
        local exit_code=$(echo "$status" | awk '{print $2}')
        
        if [[ "$pid" == "-" ]]; then
            if [[ "$exit_code" == "0" ]]; then
                echo "stopped"
            else
                echo "error:$exit_code"
            fi
        else
            echo "running:$pid"
        fi
    fi
}

function find_plist_file() {
    local service=$1
    
    for path in "${SERVICE_PATHS[@]}"; do
        if [[ -f "$path/$service.plist" ]]; then
            echo "$path/$service.plist"
            return 0
        fi
    done
    
    return 1
}

function get_service_info() {
    local service=$1
    local plist_file=$(find_plist_file "$service" 2>/dev/null || echo "")
    
    if [[ -n "$plist_file" ]]; then
        # Extract key information from plist
        local program=$(/usr/libexec/PlistBuddy -c "Print :Program" "$plist_file" 2>/dev/null || \
                       /usr/libexec/PlistBuddy -c "Print :ProgramArguments:0" "$plist_file" 2>/dev/null || \
                       echo "unknown")
        local working_dir=$(/usr/libexec/PlistBuddy -c "Print :WorkingDirectory" "$plist_file" 2>/dev/null || echo "none")
        local run_at_load=$(/usr/libexec/PlistBuddy -c "Print :RunAtLoad" "$plist_file" 2>/dev/null || echo "false")
        
        echo "plist:$plist_file|program:$program|workdir:$working_dir|autostart:$run_at_load"
    else
        echo "plist:not-found|program:unknown|workdir:none|autostart:false"
    fi
}

function list_services() {
    print_header
    echo ""
    echo -e "${BOLD}${GREEN}ACTIVE SERVICES:${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    printf "${BOLD}%-40s %-12s %-10s %-30s${NC}\n" "SERVICE" "STATUS" "PID" "DEPLOYMENT"
    echo -e "${CYAN}────────────────────────────────────────────────────────────────────${NC}"
    
    local found_services=()
    
    # Find all matching services
    for pattern in "${SERVICE_PATTERNS[@]}"; do
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                local service=$(echo "$line" | awk '{print $3}')
                found_services+=("$service")
            fi
        done < <(launchctl list | grep "$pattern" 2>/dev/null || true)
    done
    
    # Remove duplicates
    IFS=$'\n' found_services=($(sort -u <<<"${found_services[*]}")); unset IFS
    
    # Display service information
    for service in "${found_services[@]}"; do
        local status=$(get_service_status "$service")
        local info=$(get_service_info "$service")
        local plist=$(echo "$info" | grep -o 'plist:[^|]*' | cut -d: -f2)
        
        local status_display=""
        local pid_display="-"
        
        if [[ "$status" == running:* ]]; then
            pid_display=$(echo "$status" | cut -d: -f2)
            status_display="${GREEN}● running${NC}"
        elif [[ "$status" == "stopped" ]]; then
            status_display="${YELLOW}○ stopped${NC}"
        elif [[ "$status" == error:* ]]; then
            local exit_code=$(echo "$status" | cut -d: -f2)
            status_display="${RED}✗ error($exit_code)${NC}"
        else
            status_display="${RED}◌ not-loaded${NC}"
        fi
        
        # Shorten plist path for display
        local deploy_path="$plist"
        if [[ "$deploy_path" == "$HOME"* ]]; then
            deploy_path="~${deploy_path#$HOME}"
        fi
        deploy_path=$(basename "$deploy_path" .plist)
        
        printf "%-40s ${status_display}    %-10s %-30s\n" "$service" "$pid_display" "$deploy_path"
    done
    
    echo ""
}

function show_details() {
    local service=$1
    
    print_header
    echo ""
    echo -e "${BOLD}${GREEN}SERVICE DETAILS: ${YELLOW}$service${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    local status=$(get_service_status "$service")
    local info=$(get_service_info "$service")
    
    # Parse info
    local plist=$(echo "$info" | grep -o 'plist:[^|]*' | cut -d: -f2)
    local program=$(echo "$info" | grep -o 'program:[^|]*' | cut -d: -f2)
    local workdir=$(echo "$info" | grep -o 'workdir:[^|]*' | cut -d: -f2)
    local autostart=$(echo "$info" | grep -o 'autostart:[^|]*' | cut -d: -f2)
    
    echo -e "${BOLD}Status:${NC}"
    if [[ "$status" == running:* ]]; then
        local pid=$(echo "$status" | cut -d: -f2)
        echo -e "  ${GREEN}● Running${NC} (PID: $pid)"
    elif [[ "$status" == "stopped" ]]; then
        echo -e "  ${YELLOW}○ Stopped${NC}"
    else
        echo -e "  ${RED}✗ Not Loaded or Error${NC}"
    fi
    
    echo ""
    echo -e "${BOLD}Configuration:${NC}"
    echo -e "  Plist:      $plist"
    echo -e "  Program:    $program"
    echo -e "  Work Dir:   $workdir"
    echo -e "  Auto Start: $autostart"
    
    if [[ -n "$plist" ]] && [[ "$plist" != "not-found" ]]; then
        echo ""
        echo -e "${BOLD}Log Files:${NC}"
        local stdout_log=$(/usr/libexec/PlistBuddy -c "Print :StandardOutPath" "$plist" 2>/dev/null || echo "")
        local stderr_log=$(/usr/libexec/PlistBuddy -c "Print :StandardErrorPath" "$plist" 2>/dev/null || echo "")
        
        if [[ -n "$stdout_log" ]]; then
            echo -e "  Stdout: $stdout_log"
            if [[ -f "$stdout_log" ]]; then
                echo -e "          ${CYAN}($(wc -l < "$stdout_log") lines, $(du -h "$stdout_log" | awk '{print $1}'))${NC}"
            fi
        fi
        
        if [[ -n "$stderr_log" ]]; then
            echo -e "  Stderr: $stderr_log"
            if [[ -f "$stderr_log" ]]; then
                echo -e "          ${CYAN}($(wc -l < "$stderr_log") lines, $(du -h "$stderr_log" | awk '{print $1}'))${NC}"
            fi
        fi
    fi
    
    echo ""
    echo -e "${BOLD}Commands:${NC}"
    echo -e "  Start:   launchctl start $service"
    echo -e "  Stop:    launchctl stop $service"
    echo -e "  Restart: launchctl stop $service && launchctl start $service"
    echo -e "  Logs:    ./service-manager.sh logs $service"
    
    echo ""
}

function show_logs() {
    local service=$1
    local lines=${2:-50}
    
    local info=$(get_service_info "$service")
    local plist=$(echo "$info" | grep -o 'plist:[^|]*' | cut -d: -f2)
    
    if [[ -n "$plist" ]] && [[ "$plist" != "not-found" ]]; then
        local stdout_log=$(/usr/libexec/PlistBuddy -c "Print :StandardOutPath" "$plist" 2>/dev/null || echo "")
        local stderr_log=$(/usr/libexec/PlistBuddy -c "Print :StandardErrorPath" "$plist" 2>/dev/null || echo "")
        
        if [[ -n "$stdout_log" ]] && [[ -f "$stdout_log" ]]; then
            echo -e "${BOLD}${GREEN}==> STDOUT ($stdout_log) <==${NC}"
            tail -n "$lines" "$stdout_log"
            echo ""
        fi
        
        if [[ -n "$stderr_log" ]] && [[ -f "$stderr_log" ]]; then
            echo -e "${BOLD}${RED}==> STDERR ($stderr_log) <==${NC}"
            tail -n "$lines" "$stderr_log"
        fi
    else
        echo -e "${RED}No log files found for service: $service${NC}"
    fi
}

function follow_logs() {
    local service=$1
    
    local info=$(get_service_info "$service")
    local plist=$(echo "$info" | grep -o 'plist:[^|]*' | cut -d: -f2)
    
    if [[ -n "$plist" ]] && [[ "$plist" != "not-found" ]]; then
        local stdout_log=$(/usr/libexec/PlistBuddy -c "Print :StandardOutPath" "$plist" 2>/dev/null || echo "")
        local stderr_log=$(/usr/libexec/PlistBuddy -c "Print :StandardErrorPath" "$plist" 2>/dev/null || echo "")
        
        local logs_to_follow=""
        [[ -f "$stdout_log" ]] && logs_to_follow="$logs_to_follow $stdout_log"
        [[ -f "$stderr_log" ]] && logs_to_follow="$logs_to_follow $stderr_log"
        
        if [[ -n "$logs_to_follow" ]]; then
            echo -e "${BOLD}${GREEN}Following logs for: $service${NC}"
            echo -e "${CYAN}Press Ctrl+C to exit${NC}"
            echo ""
            tail -f $logs_to_follow
        else
            echo -e "${RED}No log files found for service: $service${NC}"
        fi
    else
        echo -e "${RED}Service not found: $service${NC}"
    fi
}

function show_health() {
    print_header
    echo ""
    echo -e "${BOLD}${GREEN}SYSTEM HEALTH CHECK:${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Check Ollama
    echo -e "\n${BOLD}Ollama Status:${NC}"
    if curl -s http://localhost:11434/api/version > /dev/null 2>&1; then
        local version=$(curl -s http://localhost:11434/api/version | grep -o '"version":"[^"]*"' | cut -d'"' -f4)
        echo -e "  ${GREEN}● Online${NC} (version: $version)"
        local models=$(curl -s http://localhost:11434/api/tags | grep -o '"name":"[^"]*"' | wc -l | tr -d ' ')
        echo -e "  Models loaded: $models"
    else
        echo -e "  ${RED}✗ Offline${NC}"
    fi
    
    # Check GameCode Web
    echo -e "\n${BOLD}GameCode Web:${NC}"
    if curl -s http://localhost:8080 > /dev/null 2>&1; then
        echo -e "  ${GREEN}● Online${NC} (http://localhost:8080)"
    else
        echo -e "  ${RED}✗ Offline${NC}"
    fi
    
    # Check disk usage for logs
    echo -e "\n${BOLD}Log Storage:${NC}"
    echo -e "  /tmp: $(du -sh /tmp 2>/dev/null | awk '{print $1}' || echo 'N/A')"
    echo -e "  ~/Library/Logs: $(du -sh ~/Library/Logs 2>/dev/null | awk '{print $1}' || echo 'N/A')"
    
    echo ""
}

function show_help() {
    print_header
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  list, ls          List all custom services"
    echo "  details <service> Show detailed info about a service"
    echo "  logs <service>    Show recent logs (last 50 lines)"
    echo "  follow <service>  Follow logs in real-time"
    echo "  health            System health check"
    echo "  help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 list"
    echo "  $0 details com.gamecode.web"
    echo "  $0 logs com.gamecode.web"
    echo "  $0 follow com.gamecode.web"
    echo ""
}

# Main command handling
case "${1:-help}" in
    list|ls)
        list_services
        ;;
    details)
        if [[ -z "${2:-}" ]]; then
            echo -e "${RED}Error: Service name required${NC}"
            echo "Usage: $0 details <service-name>"
            exit 1
        fi
        show_details "$2"
        ;;
    logs)
        if [[ -z "${2:-}" ]]; then
            echo -e "${RED}Error: Service name required${NC}"
            echo "Usage: $0 logs <service-name>"
            exit 1
        fi
        show_logs "$2" "${3:-50}"
        ;;
    follow)
        if [[ -z "${2:-}" ]]; then
            echo -e "${RED}Error: Service name required${NC}"
            echo "Usage: $0 follow <service-name>"
            exit 1
        fi
        follow_logs "$2"
        ;;
    health)
        show_health
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        show_help
        exit 1
        ;;
esac