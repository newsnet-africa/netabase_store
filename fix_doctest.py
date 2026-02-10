#!/usr/bin/env python3
"""
Fix doctests by adding Serialize/Deserialize derives to struct declarations.
"""

import re
import sys

def fix_doctest(content):
    """Add Serialize/Deserialize to NetabaseModel derives if missing."""
    
    # Pattern to find NetabaseModel derives without Serialize/Deserialize
    pattern = r'(#\[derive\([^)]*netabase_macros::NetabaseModel[^)]*)\]'
    
    def add_derives(match):
        derives_str = match.group(1)
        # Check if Serialize and Deserialize are already there
        if 'Serialize' not in derives_str or 'Deserialize' not in derives_str:
            # Add them after NetabaseModel
            new_derives = derives_str.replace(
                'netabase_macros::NetabaseModel',
                'netabase_macros::NetabaseModel, Serialize, Deserialize'
            )
            # Remove duplicates if they exist
            parts = new_derives.split(',')
            seen = set()
            unique = []
            for part in parts:
                clean = part.strip()
                if clean and clean not in seen:
                    seen.add(clean)
                    unique.append(part)
            return '(#[derive(' + ','.join(unique) + ')]'
        return match.group(0) + ']'
    
    return re.sub(pattern, add_derives, content)

if __name__ == '__main__':
    import sys
    if len(sys.argv) > 1:
        filename = sys.argv[1]
        with open(filename, 'r') as f:
            content = f.read()
        fixed = fix_doctest(content)
        with open(filename, 'w') as f:
            f.write(fixed)
        print(f"Fixed {filename}")
    else:
        print("Usage: fix_doctest.py <file>")
