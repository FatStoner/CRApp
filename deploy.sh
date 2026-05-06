#!/bin/bash
set -e

echo "🚀 Starting deployment..."

VERSION=$1

# 1. Handle version argument (optional)
if [[ -n "$VERSION" ]]; then
    # Validate semantic version format (x.y.z)
    if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "❌ Error: Invalid version format!"
        echo "Version must be in semantic format: x.y.z (e.g., 1.2.3)"
        exit 1
    fi

    echo "📌 Version: $VERSION"

    # 2. Update Cargo.toml version
    echo "✏️  Updating Cargo.toml version to $VERSION..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS requires empty string after -i
        sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    fi

    # Verify the change
    CARGO_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
    if [[ "$CARGO_VERSION" != "$VERSION" ]]; then
        echo "❌ Error: Failed to update Cargo.toml version!"
        exit 1
    fi
    echo "✅ Cargo.toml version updated to $CARGO_VERSION"
else
    echo "ℹ️  No version specified - creating pre-release only"
fi

# 3. Build for Windows using MinGW
echo "📦 Building for Windows (x86_64-pc-windows-gnu)..."
cargo build --release --target x86_64-pc-windows-gnu

# 4. Check Git status and commit changes
if [[ -n $(git status -s) ]]; then
    echo "📝 Committing changes..."
    git add .
    if [[ -n "$VERSION" ]]; then
        git commit -m "chore: bump version to v$VERSION and deploy"
    else
        git commit -m "chore: deploy latest build"
    fi
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

# 5. Manage GitHub Release
# Use target-specific naming for updater compatibility
TARGET="x86_64-pc-windows-gnu"
ZIP_NAME="CRApp-${TARGET}.zip"

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

echo "✅ Deployment complete!"
if [[ -n "$VERSION" ]]; then
    echo "   Versioned: https://github.com/JustJam-Dev/CRApp/releases/tag/v$VERSION"
fi
echo "   Latest: https://github.com/JustJam-Dev/CRApp/releases/tag/latest"

# Cleanup
rm -rf dist
rm $ZIP_NAME
