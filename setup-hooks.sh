#!/bin/bash
# Setup script for BeBytes git hooks

set -e

echo "Setting up git hooks for BeBytes..."

# Function to create symlink with overwrite option
create_symlink() {
    local source=$1
    local target=$2
    
    if [ -e "$target" ] || [ -L "$target" ]; then
        echo "⚠️  Hook already exists: $target"
        read -p "Overwrite? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -f "$target"
            ln -s "$source" "$target"
            echo "✅ Installed: $target"
        else
            echo "⏭️  Skipped: $target"
        fi
    else
        ln -s "$source" "$target"
        echo "✅ Installed: $target"
    fi
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d ".githooks" ]; then
    echo "❌ Error: This script must be run from the BeBytes project root directory"
    exit 1
fi

# Create .git/hooks directory if it doesn't exist
mkdir -p .git/hooks

echo
echo "Choose your setup:"
echo "1) Fast checks only (formatting + clippy + compile check) - Recommended for development"
echo "2) Full CI validation (all tests, both std and no_std) - Recommended for final commits"
echo "3) Custom (you choose which hooks to install)"
echo "4) Install pre-commit framework (requires Python)"
read -p "Enter your choice (1-4): " choice

case $choice in
    1)
        echo "Installing fast pre-commit hook..."
        create_symlink "../../.githooks/pre-commit-fast" ".git/hooks/pre-commit"
        echo
        echo "💡 You can run '.githooks/pre-commit' manually for full validation"
        ;;
    2)
        echo "Installing full validation hooks..."
        create_symlink "../../.githooks/pre-commit" ".git/hooks/pre-commit"
        create_symlink "../../.githooks/pre-push" ".git/hooks/pre-push"
        ;;
    3)
        echo "Custom installation:"
        read -p "Install fast pre-commit hook? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            create_symlink "../../.githooks/pre-commit-fast" ".git/hooks/pre-commit"
        fi
        
        read -p "Install full pre-commit hook? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            create_symlink "../../.githooks/pre-commit" ".git/hooks/pre-commit"
        fi
        
        read -p "Install pre-push hook? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            create_symlink "../../.githooks/pre-push" ".git/hooks/pre-push"
        fi
        ;;
    4)
        echo "Setting up pre-commit framework..."
        if ! command -v pre-commit &> /dev/null; then
            echo "Installing pre-commit..."
            if command -v pip &> /dev/null; then
                pip install pre-commit
            elif command -v pip3 &> /dev/null; then
                pip3 install pre-commit
            else
                echo "❌ Error: pip not found. Please install Python and pip first."
                exit 1
            fi
        fi
        pre-commit install
        echo "✅ pre-commit framework installed"
        echo "Run 'pre-commit run --all-files' to test it"
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

echo
echo "✅ Git hooks setup complete!"
echo
echo "Available hooks in .githooks/:"
echo "  - pre-commit-fast: Quick validation (fmt, clippy, check)"
echo "  - pre-commit: Full CI validation"  
echo "  - pre-push: Full validation before push"
echo
echo "You can also configure git to use the .githooks directory:"
echo "  git config core.hooksPath .githooks"