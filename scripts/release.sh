#!/bin/bash
# Release script for asyncapi-rust workspace
# Uses git-cliff to generate changelog and suggest version bump

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Running pre-release checks...${NC}"
echo ""

# Check formatting
echo -e "${BLUE}📝 Checking code formatting...${NC}"
if ! cargo fmt --all -- --check; then
    echo ""
    echo -e "${RED}❌ Formatting check failed!${NC}"
    echo -e "${YELLOW}Run: cargo fmt --all${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Formatting check passed${NC}"
echo ""

# Check clippy
echo -e "${BLUE}🔎 Running clippy...${NC}"
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo ""
    echo -e "${RED}❌ Clippy check failed!${NC}"
    echo "Fix the warnings above before releasing."
    exit 1
fi
echo -e "${GREEN}✅ Clippy check passed${NC}"
echo ""

# Run tests
echo -e "${BLUE}🧪 Running tests...${NC}"
if ! cargo test --all-features; then
    echo ""
    echo -e "${RED}❌ Tests failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ All tests passed${NC}"
echo ""

# Build documentation
echo -e "${BLUE}📚 Checking documentation...${NC}"
if ! cargo doc --all-features --no-deps; then
    echo ""
    echo -e "${RED}❌ Documentation build failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Documentation builds successfully${NC}"
echo ""

echo -e "${GREEN}🎉 All pre-release checks passed!${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${RED}Error: Must be on main branch (currently on $CURRENT_BRANCH)${NC}"
    exit 1
fi

# Check no uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}Error: Uncommitted changes exist${NC}"
    git status --short
    exit 1
fi

# Check git-cliff is installed
if ! command -v git-cliff &> /dev/null; then
    echo -e "${RED}Error: git-cliff not installed${NC}"
    echo "Install with: cargo install git-cliff"
    exit 1
fi

# Get current version
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${GREEN}Current version: v$CURRENT_VERSION${NC}"

# Get suggested version from git-cliff
echo ""
echo "Analyzing commits since v$CURRENT_VERSION..."
SUGGESTED_VERSION=$(git-cliff --bump --unreleased 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)

if [ -z "$SUGGESTED_VERSION" ]; then
    echo -e "${RED}Error: Could not determine next version${NC}"
    echo "Are there any new commits since v$CURRENT_VERSION?"
    exit 1
fi

echo -e "${YELLOW}Suggested version: v$SUGGESTED_VERSION${NC}"
echo ""

# Show unreleased changes
echo "Unreleased changes:"
echo "==================="
git-cliff --unreleased
echo ""

# Confirm with user
read -p "$(echo -e ${YELLOW}Use version v$SUGGESTED_VERSION? [y/N]:${NC} )" -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Release cancelled"
    exit 1
fi

# Update workspace version
echo ""
echo "Updating Cargo.toml..."
sed -i '' "s/^version = \".*\"/version = \"$SUGGESTED_VERSION\"/" Cargo.toml

# Update dependency versions in main crate
echo "Updating asyncapi-rust/Cargo.toml dependencies..."
sed -i '' "s/asyncapi-rust-codegen = { version = \"[^\"]*\"/asyncapi-rust-codegen = { version = \"$SUGGESTED_VERSION\"/" asyncapi-rust/Cargo.toml
sed -i '' "s/asyncapi-rust-models = { version = \"[^\"]*\"/asyncapi-rust-models = { version = \"$SUGGESTED_VERSION\"/" asyncapi-rust/Cargo.toml

# Update lock file
echo "Updating Cargo.lock..."
cargo check --quiet

# Generate changelog
echo "Generating CHANGELOG.md..."
if [ ! -f CHANGELOG.md ]; then
    # First time - generate full changelog
    git-cliff -o CHANGELOG.md
else
    # Prepend new release to existing changelog
    git-cliff --unreleased --tag "v$SUGGESTED_VERSION" --prepend CHANGELOG.md
fi

# Show diff
echo ""
echo "Changes to be committed:"
echo "======================="
git diff Cargo.toml asyncapi-rust/Cargo.toml CHANGELOG.md | head -50
echo ""

# Confirm commit
read -p "$(echo -e ${YELLOW}Commit these changes? [y/N]:${NC} )" -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Rolling back changes..."
    git checkout Cargo.toml asyncapi-rust/Cargo.toml Cargo.lock CHANGELOG.md
    echo "Release cancelled"
    exit 1
fi

# Commit
echo ""
echo "Committing changes..."
git add Cargo.toml Cargo.lock asyncapi-rust/Cargo.toml CHANGELOG.md
git commit -m "chore(release): prepare for v$SUGGESTED_VERSION"

# Tag
echo "Creating tag v$SUGGESTED_VERSION..."
git tag -a "v$SUGGESTED_VERSION" -m "Release v$SUGGESTED_VERSION"

echo ""
echo -e "${GREEN}✅ Release v$SUGGESTED_VERSION prepared!${NC}"
echo ""
echo "Next steps:"
echo "==========="
echo "1. Review the changes:"
echo "   git show HEAD"
echo ""
echo "2. Push to GitHub:"
echo "   git push origin main && git push origin v$SUGGESTED_VERSION"
echo ""
echo "3. Publish to crates.io (in dependency order):"
echo "   cd asyncapi-rust-models && cargo publish"
echo "   sleep 30  # Wait for crates.io indexing"
echo "   cd ../asyncapi-rust-codegen && cargo publish"
echo "   sleep 30"
echo "   cd ../asyncapi-rust && cargo publish"
echo ""
echo "4. Create GitHub release:"
echo "   git-cliff --latest --strip header | \\"
echo "     gh release create v$SUGGESTED_VERSION --title \"v$SUGGESTED_VERSION\" --notes-file -"
echo ""
echo "Or to undo (before pushing):"
echo "   git reset --hard HEAD~1"
echo "   git tag -d v$SUGGESTED_VERSION"
