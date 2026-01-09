#!/bin/bash
set -e

echo "🚀 Starting deployment..."

# 1. Build for Windows using MinGW
echo "📦 Building for Windows (x86_64-pc-windows-gnu)..."
cargo build --release --target x86_64-pc-windows-gnu

# 2. Check Git status and commit changes
if [[ -n $(git status -s) ]]; then
    echo "📝 Committing changes..."
    git add .
    git commit -m "chore: deploy latest build"
    echo "⬇️ Pulling latest changes..."
    git pull --rebase origin main
    echo "⬆️ Pushing to main..."
    git push origin main
else
    echo "✅ No changes to commit."
    echo "⬇️ Pulling latest changes..."
    git pull --rebase origin main
    echo "⬆️ Pushing to main..."
    git push origin main
fi

# 3. Manage GitHub Release
echo "🏷️ Updating 'latest' release..."

# Delete existing tag/release if it exists (ignore errors)
gh release delete latest --yes || true
git tag -d latest || true
git push origin :refs/tags/latest || true

# Create new release with the executable
# Using "latest" as the tag name
echo "☁️ Uploading release..."
echo "Auto-generated release from local build." | gh release create latest \
    "target/x86_64-pc-windows-gnu/release/crap.exe#crap.exe" \
    --title "Latest Build" \
    --notes-file - \
    --prerelease \
    --target main

echo "✅ Deployment complete! Download at: https://github.com/JustJam-Dev/CRApp/releases/tag/latest"
