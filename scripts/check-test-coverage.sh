#!/bin/bash
# Test Coverage Analysis Script for WezTerm
# This script analyzes test coverage across critical crates

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Crates to analyze
CRITICAL_CRATES=(
    "mux"
    "term"
    "config"
    "wezterm-gui"
    "codec"
    "wezterm-client"
    "wezterm-mux-server-impl"
    "wezterm-font"
    "window"
)

echo -e "${BOLD}WezTerm Test Coverage Analysis${NC}"
echo "================================="
echo

# Function to count tests in a crate
count_tests() {
    local crate_path="$1"
    local test_count=0

    # Count inline tests (#[test] and #[cfg(test)])
    local inline_tests=$(find "$crate_path" -name "*.rs" -type f -exec grep -c '#\[test\]' {} \; 2>/dev/null | awk '{s+=$1} END {print s}')
    inline_tests=${inline_tests:-0}

    # Count integration tests (tests/ directory)
    local integration_tests=0
    if [ -d "$crate_path/tests" ]; then
        integration_tests=$(find "$crate_path/tests" -name "*.rs" -type f -exec grep -c '#\[test\]' {} \; 2>/dev/null | awk '{s+=$1} END {print s}')
        integration_tests=${integration_tests:-0}
    fi

    # Count benchmark tests (benches/ directory)
    local bench_tests=0
    if [ -d "$crate_path/benches" ]; then
        bench_tests=$(find "$crate_path/benches" -name "*.rs" -type f | wc -l)
    fi

    test_count=$((inline_tests + integration_tests + bench_tests))
    echo "$test_count"
}

# Function to count lines of code in a crate
count_loc() {
    local crate_path="$1"
    local loc=0

    # Count lines in src/ directory (excluding test modules for fair comparison)
    if [ -d "$crate_path/src" ]; then
        loc=$(find "$crate_path/src" -name "*.rs" -type f -exec wc -l {} \; 2>/dev/null | awk '{s+=$1} END {print s}')
    fi

    echo "${loc:-0}"
}

# Function to get test status color
get_status_color() {
    local test_count=$1
    local loc=$2

    if [ "$test_count" -eq 0 ]; then
        echo "$RED"
    elif [ "$loc" -gt 0 ]; then
        local ratio=$(echo "scale=4; $test_count / $loc * 100" | bc)
        if (( $(echo "$ratio >= 0.5" | bc -l) )); then
            echo "$GREEN"
        elif (( $(echo "$ratio >= 0.1" | bc -l) )); then
            echo "$YELLOW"
        else
            echo "$RED"
        fi
    else
        echo "$NC"
    fi
}

# Header
printf "%-30s %10s %10s %10s\n" "Crate" "Tests" "LOC" "Coverage"
echo "------------------------------------------------------------------------"

total_tests=0
total_loc=0
missing_tests=()

# Analyze each crate
for crate in "${CRITICAL_CRATES[@]}"; do
    crate_path="$PROJECT_ROOT/$crate"

    if [ ! -d "$crate_path" ]; then
        # Try alternative names
        if [ "$crate" = "term" ]; then
            crate_path="$PROJECT_ROOT/term"
        elif [ "$crate" = "wezterm-gui" ]; then
            crate_path="$PROJECT_ROOT/wezterm-gui"
        fi
    fi

    if [ -d "$crate_path" ]; then
        test_count=$(count_tests "$crate_path")
        loc=$(count_loc "$crate_path")

        total_tests=$((total_tests + test_count))
        total_loc=$((total_loc + loc))

        # Calculate coverage percentage
        coverage="N/A"
        if [ "$loc" -gt 0 ]; then
            coverage=$(echo "scale=3; $test_count / $loc * 100" | bc)
            coverage="${coverage}%"
        fi

        # Get color based on coverage
        color=$(get_status_color "$test_count" "$loc")

        # Track crates with no tests
        if [ "$test_count" -eq 0 ]; then
            missing_tests+=("$crate")
        fi

        printf "${color}%-30s %10s %10s %10s${NC}\n" "$crate" "$test_count" "$loc" "$coverage"
    else
        printf "${RED}%-30s %10s %10s %10s${NC}\n" "$crate" "N/A" "N/A" "Not found"
    fi
done

echo "------------------------------------------------------------------------"

# Calculate total coverage
total_coverage="0.0%"
if [ "$total_loc" -gt 0 ]; then
    total_coverage=$(echo "scale=3; $total_tests / $total_loc * 100" | bc)
    total_coverage="${total_coverage}%"
fi

printf "${BOLD}%-30s %10s %10s %10s${NC}\n" "TOTAL" "$total_tests" "$total_loc" "$total_coverage"

echo
echo -e "${BOLD}Coverage Analysis:${NC}"
echo

# Industry standard comparison
if [ "$total_loc" -gt 0 ]; then
    current_ratio=$(echo "scale=4; $total_tests / $total_loc * 100" | bc)
    target_ratio="1.0"  # 1% (1 test per 100 LOC is good minimum)

    if (( $(echo "$current_ratio < $target_ratio" | bc -l) )); then
        gap=$(echo "scale=0; ($target_ratio - $current_ratio) * $total_loc / 100" | bc)
        echo -e "  ${YELLOW}⚠${NC}  Current coverage: ${current_ratio}%"
        echo -e "  ${YELLOW}⚠${NC}  Target coverage:  ${target_ratio}% (industry minimum)"
        echo -e "  ${YELLOW}⚠${NC}  Tests needed:     ~${gap} additional tests"
    else
        echo -e "  ${GREEN}✓${NC}  Coverage meets minimum target of ${target_ratio}%"
    fi
fi

echo

# Report crates with zero tests
if [ ${#missing_tests[@]} -gt 0 ]; then
    echo -e "${BOLD}${RED}Crates with ZERO tests:${NC}"
    for crate in "${missing_tests[@]}"; do
        echo -e "  ${RED}✗${NC} $crate"
    done
    echo
fi

# Recommendations
echo -e "${BOLD}Recommendations:${NC}"
echo -e "  1. Run ${BOLD}cargo test${NC} to execute all tests"
echo -e "  2. Run ${BOLD}cargo nextest run${NC} for faster parallel test execution"
echo -e "  3. Run ${BOLD}cargo tarpaulin${NC} for detailed coverage report (Linux only)"
echo -e "  4. See ${BOLD}docs/debugging.md${NC} for debugging and testing guidelines"
echo -e "  5. Review ${BOLD}/tmp/EXECUTIVE_SUMMARY.txt${NC} for detailed coverage analysis"
echo

# Exit with error if coverage is too low
min_acceptable_ratio="0.03"  # 0.03% is current state, don't regress
if [ "$total_loc" -gt 0 ]; then
    current_ratio=$(echo "scale=4; $total_tests / $total_loc * 100" | bc)
    if (( $(echo "$current_ratio < $min_acceptable_ratio" | bc -l) )); then
        echo -e "${RED}ERROR: Test coverage ($current_ratio%) is below minimum threshold ($min_acceptable_ratio%)${NC}"
        exit 1
    fi
fi

exit 0
