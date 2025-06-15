#!/bin/bash
# DataFold Configuration Migration Script
# Migrates legacy configuration files to the new cross-platform system

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKUP_DIR="${HOME}/.config/datafold/backups/$(date +%Y%m%d_%H%M%S)"

# Logging
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if datafold_cli is available
check_cli() {
    if ! command -v datafold_cli &> /dev/null; then
        log_error "datafold_cli not found. Please build and install the CLI first:"
        echo "  cargo build --release --workspace"
        echo "  sudo cp target/release/datafold_cli /usr/local/bin/"
        exit 1
    fi
}

# Create backup directory
create_backup_dir() {
    log_info "Creating backup directory: $BACKUP_DIR"
    mkdir -p "$BACKUP_DIR"
}

# Backup existing configurations
backup_configs() {
    log_info "Backing up existing configurations..."
    
    # Find configuration files
    local config_files=(
        "$HOME/.config/datafold/cli_config.json"
        "$HOME/.config/datafold/config.json"
        "$HOME/.config/datafold/unified-datafold-config.json"
        "$HOME/.config/datafold/logging_config.toml"
        "$HOME/.config/datafold/node_config.json"
    )
    
    local backed_up=0
    for config_file in "${config_files[@]}"; do
        if [[ -f "$config_file" ]]; then
            log_info "Backing up: $(basename "$config_file")"
            cp "$config_file" "$BACKUP_DIR/"
            ((backed_up++))
        fi
    done
    
    if [[ $backed_up -eq 0 ]]; then
        log_warning "No legacy configuration files found to backup"
    else
        log_success "Backed up $backed_up configuration files to $BACKUP_DIR"
    fi
}

# Run migration
run_migration() {
    log_info "Starting configuration migration..."
    
    # Use the built-in migration tool
    if datafold_cli migrate --all --backup; then
        log_success "Configuration migration completed successfully"
    else
        log_error "Configuration migration failed"
        log_info "Check the logs and backup files in: $BACKUP_DIR"
        return 1
    fi
}

# Validate migrated configuration
validate_config() {
    log_info "Validating migrated configuration..."
    
    if datafold_cli config validate; then
        log_success "Configuration validation passed"
    else
        log_error "Configuration validation failed"
        log_info "You may need to manually fix configuration issues"
        return 1
    fi
}

# Test configuration loading
test_config() {
    log_info "Testing configuration loading..."
    
    if datafold_cli config show > /dev/null; then
        log_success "Configuration loads successfully"
    else
        log_error "Failed to load configuration"
        return 1
    fi
}

# Generate migration report
generate_report() {
    local report_file="$BACKUP_DIR/migration_report.txt"
    log_info "Generating migration report: $report_file"
    
    cat > "$report_file" << EOF
DataFold Configuration Migration Report
======================================
Date: $(date)
Migration Directory: $BACKUP_DIR
Script Version: 1.0

Migration Summary:
- Legacy configurations backed up to: $BACKUP_DIR
- New configuration location: ~/.config/datafold/config.toml
- Migration method: Automated using datafold_cli migrate

Platform Information:
- OS: $(uname -s)
- Architecture: $(uname -m)
- User: $(whoami)
- Home: $HOME

Configuration Status:
EOF
    
    if datafold_cli config validate &>> "$report_file"; then
        echo "- Validation: PASSED" >> "$report_file"
    else
        echo "- Validation: FAILED" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "For support, see: docs/config/deployment.md" >> "$report_file"
    
    log_success "Migration report saved to: $report_file"
}

# Main migration function
main() {
    echo "DataFold Configuration Migration Tool"
    echo "====================================="
    echo ""
    
    # Pre-flight checks
    check_cli
    
    # Create backup directory
    create_backup_dir
    
    # Backup existing configurations
    backup_configs
    
    # Run migration
    if ! run_migration; then
        log_error "Migration failed. Check backup files in: $BACKUP_DIR"
        exit 1
    fi
    
    # Validate migrated configuration
    if ! validate_config; then
        log_warning "Configuration validation failed. Manual intervention may be required."
    fi
    
    # Test configuration loading
    if ! test_config; then
        log_warning "Configuration loading test failed. Check the configuration manually."
    fi
    
    # Generate report
    generate_report
    
    echo ""
    log_success "Migration completed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Review the migrated configuration: datafold_cli config show"
    echo "2. Test your applications with the new configuration"
    echo "3. Remove backup files once you're satisfied: rm -rf $BACKUP_DIR"
    echo "4. See docs/config/ for detailed documentation"
    echo ""
}

# Handle command line arguments
case "${1:-help}" in
    "migrate"|"run")
        main
        ;;
    "check")
        check_cli
        log_success "CLI tool is available and ready for migration"
        ;;
    "backup-only")
        create_backup_dir
        backup_configs
        ;;
    "validate")
        validate_config
        ;;
    "help"|*)
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  migrate      Run complete migration process (default)"
        echo "  check        Check if CLI tool is available"
        echo "  backup-only  Only backup existing configurations"
        echo "  validate     Validate current configuration"
        echo "  help         Show this help message"
        echo ""
        echo "For detailed documentation, see: docs/config/deployment.md"
        ;;
esac