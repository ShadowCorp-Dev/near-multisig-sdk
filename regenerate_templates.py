#!/usr/bin/env python3
import re

# Read template files
with open('templates/basic/contract/src/lib.rs') as f:
    basic_lib = f.read()

with open('templates/timelock/contract/src/lib.rs') as f:
    timelock_lib = f.read()

with open('templates/weighted/contract/src/lib.rs') as f:
    weighted_lib = f.read()

# Read CLI file
with open('cli/src/commands/init.rs') as f:
    cli_content = f.read()

# Replace embedded templates
cli_content = re.sub(
    r'const BASIC_TEMPLATE_LIB: &str = r#".*?"#;',
    f'const BASIC_TEMPLATE_LIB: &str = r#"{basic_lib}"#;',
    cli_content,
    flags=re.DOTALL
)

cli_content = re.sub(
    r'const TIMELOCK_TEMPLATE_LIB: &str = r#".*?"#;',
    f'const TIMELOCK_TEMPLATE_LIB: &str = r#"{timelock_lib}"#;',
    cli_content,
    flags=re.DOTALL
)

cli_content = re.sub(
    r'const WEIGHTED_TEMPLATE_LIB: &str = r#".*?"#;',
    f'const WEIGHTED_TEMPLATE_LIB: &str = r#"{weighted_lib}"#;',
    cli_content,
    flags=re.DOTALL
)

# Write back
with open('cli/src/commands/init.rs', 'w') as f:
    f.write(cli_content)

print("✅ CLI templates regenerated successfully!")
print(f"  Basic template: {len(basic_lib)} bytes")
print(f"  Timelock template: {len(timelock_lib)} bytes")
print(f"  Weighted template: {len(weighted_lib)} bytes")
