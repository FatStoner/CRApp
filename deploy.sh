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
VERSION=$1
ZIP_NAME="CRApp-release.zip"

echo "📦 Packaging release..."
# Create a temporary distribution directory
rm -rf dist
mkdir -p dist/data/background
mkdir -p dist/data/dictionaries

# Copy executable
cp target/x86_64-pc-windows-gnu/release/crap.exe dist/

# Copy assets
cp data/background/default.png dist/data/background/
cp -r data/dictionaries/* dist/data/dictionaries/

# Create ZIP archive
cd dist
zip -r ../$ZIP_NAME .
cd ..

if [[ -n "$VERSION" ]]; then
    echo "🏷️ Creating versioned release: v$VERSION"
    echo "Release v$VERSION" | gh release create "v$VERSION" \
        "$ZIP_NAME#$ZIP_NAME" \
        --title "v$VERSION" \
        --notes-file - \
        --target main
    echo "✅ Version v$VERSION published!"
fi

echo "🏷️ Updating 'latest' release..."

# Delete existing tag/release if it exists (ignore errors)
gh release delete latest --yes || true
git tag -d latest || true
git push origin :refs/tags/latest || true

# Create new release with the zip
# Using "latest" as the tag name
echo "☁️ Uploading release..."
echo "Auto-generated release from local build." | gh release create latest \
    "$ZIP_NAME#$ZIP_NAME" \
    --title "Latest Build" \
    --notes-file - \
    --prerelease \
    --target main

echo "✅ Deployment complete! Download at: https://github.com/JustJam-Dev/CRApp/releases/tag/latest"
if [[ -n "$VERSION" ]]; then
    echo "   And version: https://github.com/JustJam-Dev/CRApp/releases/tag/v$VERSION"
fi

# Cleanup
rm -rf dist
rm $ZIP_NAME
