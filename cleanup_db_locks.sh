#!/bin/bash

# Script to clean up database locks for the datafold project
# This script will find and kill processes holding database locks and clean up lock files

echo "🔍 Cleaning up database locks for datafold project..."

# Function to kill processes using specific database paths
kill_db_processes() {
    local db_path="$1"
    echo "Checking for processes using database: $db_path"
    
    # Find processes using the database files
    if command -v lsof >/dev/null 2>&1; then
        # Use lsof if available (more reliable)
        local pids=$(lsof +D "$db_path" 2>/dev/null | awk 'NR>1 {print $2}' | sort -u)
        if [ -n "$pids" ]; then
            echo "Found processes using $db_path: $pids"
            for pid in $pids; do
                echo "Killing process $pid..."
                kill -TERM "$pid" 2>/dev/null || true
                sleep 1
                # Force kill if still running
                if kill -0 "$pid" 2>/dev/null; then
                    echo "Force killing process $pid..."
                    kill -KILL "$pid" 2>/dev/null || true
                fi
            done
        fi
    else
        # Fallback: kill by process name
        echo "lsof not available, using process name matching..."
        pkill -f "datafold" 2>/dev/null || true
        pkill -f "fold_node" 2>/dev/null || true
        pkill -f "cargo test" 2>/dev/null || true
    fi
}

# Function to remove lock files
remove_lock_files() {
    local db_path="$1"
    echo "Removing lock files in: $db_path"
    
    if [ -d "$db_path" ]; then
        # Remove sled lock files
        find "$db_path" -name "*.lock" -type f -delete 2>/dev/null || true
        find "$db_path" -name "lock" -type f -delete 2>/dev/null || true
        find "$db_path" -name "LOCK" -type f -delete 2>/dev/null || true
        
        # Remove sled database files if they exist (this will force a clean restart)
        # Uncomment the next lines if you want to completely reset the database
        # find "$db_path" -name "db" -type f -delete 2>/dev/null || true
        # find "$db_path" -name "*.sled" -type f -delete 2>/dev/null || true
        
        echo "Lock files removed from $db_path"
    fi
}

# Common database paths used by the project
DB_PATHS=(
    "./data"
    "./fold_node/data"
    "/tmp/datafold_test_*"
    "$(pwd)/data"
)

# Kill processes and remove locks for each path
for db_path in "${DB_PATHS[@]}"; do
    if [[ "$db_path" == *"*"* ]]; then
        # Handle wildcard paths
        for expanded_path in $db_path; do
            if [ -d "$expanded_path" ]; then
                kill_db_processes "$expanded_path"
                remove_lock_files "$expanded_path"
            fi
        done
    else
        if [ -d "$db_path" ]; then
            kill_db_processes "$db_path"
            remove_lock_files "$db_path"
        fi
    fi
done

# Also clean up any test temporary directories
echo "Cleaning up test temporary directories..."
find /tmp -name "*datafold*" -type d -exec rm -rf {} + 2>/dev/null || true
find /tmp -name "*fold_node*" -type d -exec rm -rf {} + 2>/dev/null || true

# Kill any remaining cargo test processes
echo "Killing any remaining cargo test processes..."
pkill -f "cargo.*test" 2>/dev/null || true

# Wait a moment for processes to clean up
sleep 2

echo "✅ Database lock cleanup completed!"
echo ""
echo "You can now run your tests again. If you still get lock errors, try:"
echo "1. Restart your terminal"
echo "2. Run: rm -rf ./data (this will reset your local database)"
echo "3. Check for any running datafold processes: ps aux | grep datafold"