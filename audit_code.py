import os
import sys

def audit_simple(root_dir):
    print(" Analyzing Rust codebase in: " + root_dir)
    
    stats = {
        'files': 0, 'lines': 0, 'code': 0, 'comments': 0, 'blank': 0,
        'fn': 0, 'struct': 0, 'enum': 0, 'impl': 0
    }
    
    ignore_dirs = {'.git', 'target', 'node_modules', '.crap_data', 'docs'}

    print(f"{'File':<60} | {'Lines':<8} | {'Code':<8} | {'Cmnts':<8} | {'Blank':<8}")
    print("-" * 105)

    for dirpath, dirnames, filenames in os.walk(root_dir):
        # modify dirnames in-place to skip ignored
        dirnames[:] = [d for d in dirnames if d not in ignore_dirs]
        
        for filename in filenames:
            if not filename.endswith('.rs'):
                continue
                
            filepath = os.path.join(dirpath, filename)
            rel_path = os.path.relpath(filepath, root_dir)
            
            stats['files'] += 1
            f_stats = {'lines': 0, 'code': 0, 'comments': 0, 'blank': 0}
            
            try:
                with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                    lines = f.readlines()
                    f_stats['lines'] = len(lines)
                    
                    in_block = False
                    for line in lines:
                        s = line.strip()
                        if not s:
                            f_stats['blank'] += 1
                            continue
                        
                        if s.startswith('//'):
                            f_stats['comments'] += 1
                            continue
                        
                        if s.startswith('/*'):
                            in_block = True
                        
                        if in_block:
                            f_stats['comments'] += 1
                            if '*/' in s:
                                in_block = False
                            continue
                            
                        f_stats['code'] += 1
                        
                        # Simple heuristics
                        # Check strictly if it starts with the keyword to avoid miscounting
                        # e.g. "  pub fn " or "fn "
                        valid_start = s.split(' ')
                        
                        # Very basic token check
                        if 'fn' in line: stats['fn'] += 1
                        if 'struct' in line: stats['struct'] += 1
                        if 'enum' in line: stats['enum'] += 1
                        if 'impl' in line: stats['impl'] += 1

            except Exception as e:
                print(f"Error reading {filename}: {e}")
                continue
            
            # Add file stats to total
            stats['lines'] += f_stats['lines']
            stats['code'] += f_stats['code']
            stats['comments'] += f_stats['comments']
            stats['blank'] += f_stats['blank']
            
            print(f"{rel_path:<60} | {f_stats['lines']:<8} | {f_stats['code']:<8} | {f_stats['comments']:<8} | {f_stats['blank']:<8}")

    print("-" * 105)
    print(f"TOTAL FILES: {stats['files']}")
    print(f"Total Lines: {stats['lines']}")
    print(f"  Code:      {stats['code']}")
    print(f"  Comments:  {stats['comments']}")
    print(f"  Blank:     {stats['blank']}")
    print("-" * 30)
    print("Heuristic Object Counts (approx):")
    print(f"  Functions: {stats['fn']}")
    print(f"  Structs:   {stats['struct']}")
    print(f"  Enums:     {stats['enum']}")
    print(f"  Impls:     {stats['impl']}")

if __name__ == "__main__":
    audit_simple(".")
