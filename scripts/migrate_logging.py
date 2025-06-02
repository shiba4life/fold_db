#!/usr/bin/env python3
"""
DataFold Logging Migration Script

This script helps migrate existing log statements to use the new feature-specific
logging macros provided by the DataFold logging system.

Usage:
    python scripts/migrate_logging.py /path/to/source/directory
    python scripts/migrate_logging.py --file /path/to/specific/file.rs
    python scripts/migrate_logging.py --dry-run /path/to/source/directory
    python scripts/migrate_logging.py --report /path/to/source/directory

Features:
- Detects log statements that could use feature-specific macros
- Suggests replacements based on file/module context
- Generates migration reports
- Supports dry-run mode for safe analysis
- Handles both individual files and directory trees
"""

import os
import re
import sys
import argparse
from pathlib import Path
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass
from collections import defaultdict

@dataclass
class LogStatement:
    """Represents a log statement found in source code"""
    file_path: str
    line_number: int
    original_line: str
    log_level: str
    current_macro: str
    suggested_macro: Optional[str]
    suggested_feature: Optional[str]
    confidence: float  # 0.0 to 1.0

@dataclass
class MigrationReport:
    """Migration analysis report"""
    files_analyzed: int
    total_log_statements: int
    migrable_statements: int
    suggestions_by_feature: Dict[str, int]
    high_confidence_suggestions: int
    files_with_suggestions: List[str]

class LoggingMigrator:
    """Main migration analysis and suggestion engine"""
    
    # Feature detection patterns based on file paths and content
    FEATURE_PATTERNS = {
        'transform': [
            r'transform/',
            r'transform\.rs',
            r'ast\.rs',
            r'interpreter\.rs',
            r'executor\.rs',
            r'transform_.*\.rs',
        ],
        'network': [
            r'network/',
            r'network\.rs',
            r'tcp_.*\.rs',
            r'connections\.rs',
            r'discovery\.rs',
            r'peer.*\.rs',
            r'p2p.*\.rs',
        ],
        'schema': [
            r'schema/',
            r'schema\.rs',
            r'validator\.rs',
            r'json_schema\.rs',
            r'schema_.*\.rs',
        ],
        'database': [
            r'db_operations/',
            r'database\.rs',
            r'db\.rs',
            r'storage\.rs',
            r'persistence\.rs',
        ],
        'http_server': [
            r'http_server\.rs',
            r'.*_routes\.rs',
            r'web_.*\.rs',
            r'api_.*\.rs',
        ],
        'tcp_server': [
            r'tcp_server\.rs',
            r'tcp_.*\.rs',
        ],
        'query': [
            r'query\.rs',
            r'query_.*\.rs',
        ],
        'mutation': [
            r'mutation\.rs',
            r'mutation_.*\.rs',
        ],
        'permissions': [
            r'permissions/',
            r'permission.*\.rs',
            r'auth.*\.rs',
        ],
        'ingestion': [
            r'ingestion/',
            r'ingestion\.rs',
            r'ingest.*\.rs',
        ],
    }
    
    # Content-based feature detection
    CONTENT_PATTERNS = {
        'transform': [
            r'transform',
            r'AST',
            r'parse.*expression',
            r'execute.*transform',
            r'interpreter',
        ],
        'network': [
            r'peer',
            r'connection',
            r'tcp.*stream',
            r'network',
            r'discovery',
            r'heartbeat',
        ],
        'schema': [
            r'schema',
            r'validate',
            r'json.*schema',
            r'field.*type',
        ],
        'database': [
            r'database',
            r'db.*operation',
            r'storage',
            r'persist',
            r'transaction',
        ],
        'http_server': [
            r'http',
            r'request',
            r'response',
            r'endpoint',
            r'route',
            r'api',
        ],
        'tcp_server': [
            r'tcp',
            r'socket',
            r'listener',
            r'accept',
        ],
        'query': [
            r'query',
            r'select',
            r'filter',
            r'search',
        ],
        'mutation': [
            r'mutation',
            r'insert',
            r'update',
            r'delete',
            r'modify',
        ],
        'permissions': [
            r'permission',
            r'authorize',
            r'access.*control',
            r'auth',
        ],
        'ingestion': [
            r'ingest',
            r'import',
            r'load.*data',
            r'process.*input',
        ],
    }
    
    # Standard log macros to detect
    LOG_MACROS = ['trace!', 'debug!', 'info!', 'warn!', 'error!']
    LOG_LEVELS = ['trace', 'debug', 'info', 'warn', 'error']
    
    def __init__(self):
        self.log_statements: List[LogStatement] = []
        self.feature_cache: Dict[str, Optional[str]] = {}
    
    def analyze_file(self, file_path: str) -> List[LogStatement]:
        """Analyze a single Rust file for log statements"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
        except (IOError, UnicodeDecodeError) as e:
            print(f"Warning: Could not read {file_path}: {e}")
            return []
        
        statements = []
        detected_feature = self._detect_feature(file_path, content)
        
        for line_num, line in enumerate(lines, 1):
            log_statement = self._analyze_line(file_path, line_num, line, detected_feature)
            if log_statement:
                statements.append(log_statement)
        
        return statements
    
    def _analyze_line(self, file_path: str, line_num: int, line: str, 
                     detected_feature: Optional[str]) -> Optional[LogStatement]:
        """Analyze a single line for log statements"""
        stripped_line = line.strip()
        
        # Skip comments and empty lines
        if not stripped_line or stripped_line.startswith('//'):
            return None
        
        # Check for log macros
        for macro in self.LOG_MACROS:
            if f'log::{macro}' in line:
                level = macro.replace('!', '')
                suggested_macro, confidence = self._suggest_feature_macro(
                    level, detected_feature, line
                )
                
                return LogStatement(
                    file_path=file_path,
                    line_number=line_num,
                    original_line=line,
                    log_level=level,
                    current_macro=f'log::{macro}',
                    suggested_macro=suggested_macro,
                    suggested_feature=detected_feature,
                    confidence=confidence
                )
        
        # Check for bare log macros (without log:: prefix)
        for level in self.LOG_LEVELS:
            macro = f'{level}!'
            if re.search(rf'\b{re.escape(macro)}', line):
                suggested_macro, confidence = self._suggest_feature_macro(
                    level, detected_feature, line
                )
                
                return LogStatement(
                    file_path=file_path,
                    line_number=line_num,
                    original_line=line,
                    log_level=level,
                    current_macro=macro,
                    suggested_macro=suggested_macro,
                    suggested_feature=detected_feature,
                    confidence=confidence
                )
        
        return None
    
    def _detect_feature(self, file_path: str, content: str) -> Optional[str]:
        """Detect the most likely feature for a file"""
        if file_path in self.feature_cache:
            return self.feature_cache[file_path]
        
        # Normalize path for pattern matching
        normalized_path = file_path.replace('\\', '/')
        
        feature_scores = defaultdict(float)
        
        # Score based on file path patterns
        for feature, patterns in self.FEATURE_PATTERNS.items():
            for pattern in patterns:
                if re.search(pattern, normalized_path, re.IGNORECASE):
                    feature_scores[feature] += 2.0
        
        # Score based on content patterns
        for feature, patterns in self.CONTENT_PATTERNS.items():
            for pattern in patterns:
                matches = len(re.findall(pattern, content, re.IGNORECASE))
                feature_scores[feature] += matches * 0.5
        
        # Get the highest scoring feature
        if feature_scores:
            best_feature = max(feature_scores, key=feature_scores.get)
            if feature_scores[best_feature] >= 1.0:  # Minimum threshold
                self.feature_cache[file_path] = best_feature
                return best_feature
        
        self.feature_cache[file_path] = None
        return None
    
    def _suggest_feature_macro(self, level: str, feature: Optional[str], 
                              line: str) -> Tuple[Optional[str], float]:
        """Suggest a feature-specific macro and confidence level"""
        if not feature:
            return None, 0.0
        
        # Check if already using a feature-specific macro
        feature_macros = [
            'log_transform_', 'log_network_', 'log_schema_', 'log_http_'
        ]
        
        for feature_macro in feature_macros:
            if feature_macro in line:
                return None, 0.0  # Already using feature-specific macro
        
        # Map features to available macros
        macro_mapping = {
            'transform': f'log_transform_{level}!',
            'network': f'log_network_{level}!',
            'schema': f'log_schema_{level}!',
            'http_server': f'log_http_{level}!',
        }
        
        if feature in macro_mapping:
            confidence = self._calculate_confidence(feature, line)
            return macro_mapping[feature], confidence
        
        # For features without specific macros, suggest target parameter
        suggested = f'log::{level}!(target: "datafold_node::{feature}", ...)'
        confidence = self._calculate_confidence(feature, line) * 0.8  # Lower confidence
        
        return suggested, confidence
    
    def _calculate_confidence(self, feature: str, line: str) -> float:
        """Calculate confidence score for a suggestion"""
        confidence = 0.5  # Base confidence
        
        # Check for feature-related keywords in the log message
        if feature in self.CONTENT_PATTERNS:
            for pattern in self.CONTENT_PATTERNS[feature]:
                if re.search(pattern, line, re.IGNORECASE):
                    confidence += 0.2
        
        # Higher confidence for structured logging
        if re.search(r'[a-zA-Z_]+ = ', line):
            confidence += 0.2
        
        # Lower confidence for simple string messages
        if re.search(r'^\s*\w+!\s*\(\s*"[^"]*"\s*\)\s*;?\s*$', line):
            confidence -= 0.1
        
        return min(1.0, max(0.0, confidence))
    
    def analyze_directory(self, directory: str) -> List[LogStatement]:
        """Recursively analyze all Rust files in a directory"""
        all_statements = []
        
        for root, dirs, files in os.walk(directory):
            # Skip target and build directories
            dirs[:] = [d for d in dirs if d not in ['target', 'build', '.git']]
            
            for file in files:
                if file.endswith('.rs'):
                    file_path = os.path.join(root, file)
                    statements = self.analyze_file(file_path)
                    all_statements.extend(statements)
        
        return all_statements
    
    def generate_report(self, statements: List[LogStatement]) -> MigrationReport:
        """Generate a migration report from analyzed statements"""
        migrable = [s for s in statements if s.suggested_macro]
        high_confidence = [s for s in migrable if s.confidence >= 0.7]
        
        suggestions_by_feature = defaultdict(int)
        files_with_suggestions = set()
        
        for stmt in migrable:
            if stmt.suggested_feature:
                suggestions_by_feature[stmt.suggested_feature] += 1
                files_with_suggestions.add(stmt.file_path)
        
        # Count unique files analyzed
        files_analyzed = len(set(s.file_path for s in statements))
        
        return MigrationReport(
            files_analyzed=files_analyzed,
            total_log_statements=len(statements),
            migrable_statements=len(migrable),
            suggestions_by_feature=dict(suggestions_by_feature),
            high_confidence_suggestions=len(high_confidence),
            files_with_suggestions=list(files_with_suggestions)
        )
    
    def print_report(self, report: MigrationReport):
        """Print a formatted migration report"""
        print("\n" + "="*60)
        print("DataFold Logging Migration Report")
        print("="*60)
        
        print(f"\nFiles analyzed: {report.files_analyzed}")
        print(f"Total log statements found: {report.total_log_statements}")
        print(f"Statements that could use feature macros: {report.migrable_statements}")
        print(f"High confidence suggestions: {report.high_confidence_suggestions}")
        
        if report.suggestions_by_feature:
            print(f"\nSuggestions by feature:")
            for feature, count in sorted(report.suggestions_by_feature.items()):
                print(f"  {feature}: {count} statements")
        
        if report.files_with_suggestions:
            print(f"\nFiles with migration opportunities:")
            for file_path in sorted(report.files_with_suggestions):
                print(f"  {file_path}")
    
    def print_detailed_suggestions(self, statements: List[LogStatement], 
                                 min_confidence: float = 0.5):
        """Print detailed migration suggestions"""
        migrable = [s for s in statements if s.suggested_macro and s.confidence >= min_confidence]
        
        if not migrable:
            print(f"\nNo migration suggestions found with confidence >= {min_confidence}")
            return
        
        print(f"\n" + "="*60)
        print(f"Detailed Migration Suggestions (confidence >= {min_confidence})")
        print("="*60)
        
        # Group by file
        by_file = defaultdict(list)
        for stmt in migrable:
            by_file[stmt.file_path].append(stmt)
        
        for file_path, file_statements in sorted(by_file.items()):
            print(f"\n📁 {file_path}")
            print("-" * 40)
            
            for stmt in sorted(file_statements, key=lambda x: x.line_number):
                print(f"\nLine {stmt.line_number} (confidence: {stmt.confidence:.2f})")
                print(f"Current:   {stmt.current_macro}")
                print(f"Suggested: {stmt.suggested_macro}")
                if stmt.suggested_feature:
                    print(f"Feature:   {stmt.suggested_feature}")
                print(f"Code:      {stmt.original_line.strip()}")
    
    def generate_diff_suggestions(self, statements: List[LogStatement], 
                                output_file: str):
        """Generate a file with diff-style suggestions"""
        migrable = [s for s in statements if s.suggested_macro]
        
        if not migrable:
            print("No migration suggestions to write")
            return
        
        with open(output_file, 'w') as f:
            f.write("# DataFold Logging Migration Suggestions\n")
            f.write("# Generated by migrate_logging.py\n\n")
            
            by_file = defaultdict(list)
            for stmt in migrable:
                by_file[stmt.file_path].append(stmt)
            
            for file_path, file_statements in sorted(by_file.items()):
                f.write(f"\n## {file_path}\n\n")
                
                for stmt in sorted(file_statements, key=lambda x: x.line_number):
                    f.write(f"### Line {stmt.line_number} (confidence: {stmt.confidence:.2f})\n\n")
                    f.write("```diff\n")
                    f.write(f"- {stmt.original_line.strip()}\n")
                    
                    # Generate suggested replacement
                    suggested_line = self._generate_replacement_line(stmt)
                    f.write(f"+ {suggested_line}\n")
                    f.write("```\n\n")
        
        print(f"Migration suggestions written to: {output_file}")
    
    def _generate_replacement_line(self, stmt: LogStatement) -> str:
        """Generate a replacement line for a log statement"""
        original = stmt.original_line.strip()
        
        if stmt.suggested_macro and '!' in stmt.suggested_macro:
            # Replace the macro part
            old_macro_pattern = re.escape(stmt.current_macro)
            new_macro = stmt.suggested_macro.replace('!', '')
            
            # Handle log:: prefix
            if 'log::' in original:
                old_pattern = f'log::{re.escape(stmt.log_level)}!'
                replacement = f'{new_macro}!'
            else:
                old_pattern = f'{re.escape(stmt.log_level)}!'
                replacement = f'{new_macro}!'
            
            return re.sub(old_pattern, replacement, original, count=1)
        
        return original  # Fallback

def main():
    parser = argparse.ArgumentParser(
        description="Migrate DataFold logging statements to use feature-specific macros"
    )
    parser.add_argument(
        'path',
        help='Path to analyze (file or directory)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Analyze only, do not suggest changes'
    )
    parser.add_argument(
        '--report',
        action='store_true',
        help='Generate summary report only'
    )
    parser.add_argument(
        '--detailed',
        action='store_true',
        help='Show detailed suggestions'
    )
    parser.add_argument(
        '--min-confidence',
        type=float,
        default=0.5,
        help='Minimum confidence for suggestions (0.0-1.0)'
    )
    parser.add_argument(
        '--output',
        help='Output file for suggestions'
    )
    
    args = parser.parse_args()
    
    if not os.path.exists(args.path):
        print(f"Error: Path {args.path} does not exist")
        sys.exit(1)
    
    migrator = LoggingMigrator()
    
    print(f"Analyzing logging statements in: {args.path}")
    
    # Analyze the path
    if os.path.isfile(args.path):
        statements = migrator.analyze_file(args.path)
    else:
        statements = migrator.analyze_directory(args.path)
    
    # Generate and print report
    report = migrator.generate_report(statements)
    migrator.print_report(report)
    
    # Show detailed suggestions if requested
    if args.detailed and not args.report:
        migrator.print_detailed_suggestions(statements, args.min_confidence)
    
    # Generate output file if requested
    if args.output:
        migrator.generate_diff_suggestions(statements, args.output)
    
    # Summary
    if report.migrable_statements > 0:
        print(f"\n✨ Found {report.migrable_statements} opportunities to use feature-specific logging!")
        print(f"   {report.high_confidence_suggestions} high-confidence suggestions")
        if not args.detailed:
            print("   Use --detailed to see specific suggestions")
        if not args.output:
            print("   Use --output filename.md to save suggestions to a file")
    else:
        print(f"\n✅ No migration opportunities found. Your logging is already optimized!")

if __name__ == "__main__":
    main()