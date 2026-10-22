#!/usr/bin/env python3
"""Check backend dependency boundaries and the frozen pre-refactor IPC/storage contracts."""
from pathlib import Path
import json
import re
import sys

PROJECT = Path(__file__).resolve().parents[1]
SOURCE = PROJECT / 'backend/src'
CONTRACT = PROJECT / 'backend/tests/fixtures/backend-contract.json'


def balanced_end(text, start, opening, closing):
    depth = 0
    for index in range(start, len(text)):
        if text[index] == opening:
            depth += 1
        elif text[index] == closing:
            depth -= 1
            if depth == 0:
                return index + 1
    raise ValueError('unbalanced Rust declaration')


def normalize(text):
    text = re.sub(r'//[^\n]*', '', text)
    text = text.replace('pub(crate)', '').replace('pub(super)', '')
    text = text.replace('DirectoryEntryDto', 'MarkdownFileEntryDto')
    return re.sub(r'\s+', '', text).rstrip(',')


def parameters(text):
    items, start, depth = [], 0, 0
    for index, char in enumerate(text):
        if char in '<([{':
            depth += 1
        elif char in '>)]}':
            depth -= 1
        elif char == ',' and depth == 0:
            items.append(text[start:index].strip())
            start = index + 1
    items.append(text[start:].strip())
    return [item for item in items if item]


def snapshot(source, registration):
    handler = registration.read_text()
    registered = re.findall(r'::(\w+)\s*(?:,|\])', handler[handler.index('generate_handler!'):])
    if len(registered) != len(set(registered)):
        raise ValueError('duplicate registered command')
    definitions, serialized = {}, {}
    for path in sorted(source.rglob('*.rs')):
        text = path.read_text()
        for match in re.finditer(r'#\[tauri::command\]\s*pub\(crate\)\s+async\s+fn\s+(\w+)\s*\(', text):
            name = match[1]
            start = match.end() - 1
            end = balanced_end(text, start, '(', ')')
            args = []
            for item in parameters(text[start + 1:end - 1]):
                arg, kind = item.split(':', 1)
                if 'State<' in normalize(kind) or normalize(kind) == 'AppHandle':
                    continue
                args.append(normalize(arg) + ':' + normalize(kind))
            result = text[end:text.index('{', end)]
            definitions[name] = {'arguments': args, 'result': normalize(result)}
        pattern = r'(?m)^((?:#\[[^\]]+\]\s*)+)(?:pub\(crate\)\s+)?(struct|enum)\s+(\w+)\s*\{'
        for match in re.finditer(pattern, text):
            attributes = match[1]
            if not re.search(r'\b(?:Serialize|Deserialize)\b', attributes):
                continue
            end = balanced_end(text, match.end() - 1, '{', '}')
            name = match[3].replace('DirectoryEntryDto', 'MarkdownFileEntryDto')
            serde = ''.join(re.findall(r'#\[serde\([^\]]+\)\]', attributes))
            serialized[name] = normalize(serde + text[match.end():end - 1])
    missing = sorted(set(registered) - definitions.keys())
    if missing:
        raise ValueError('registered commands missing definitions: ' + ', '.join(missing))
    return {'commands': {key: definitions[key] for key in sorted(registered)}, 'serialized_types': serialized}


def check():
    errors = []
    for old in ['application', 'commands', 'domain', 'infrastructure']:
        if (SOURCE / old).exists() or (SOURCE / (old + '.rs')).exists():
            errors.append('obsolete layer remains: ' + old)
    for path in SOURCE.rglob('*.rs'):
        relative = path.relative_to(SOURCE)
        text = path.read_text()
        # Rust private modules enforce the detailed cross-feature API boundary.
        if relative.parts[0] in {'platform', 'shared'} and re.search(r'crate::(?:features|app)::', text):
            errors.append(str(relative) + ': base module depends on business/application code')
        if relative.parts[0] == 'features' and re.search(r'crate::app::', text):
            errors.append(str(relative) + ': feature depends on application assembly')
        if relative.parts[:2] == ('features', 'search') and re.search(r'crate::features::(?:pdf|markdown)::', text):
            errors.append(str(relative) + ': search depends on a concrete document feature')
        if path.name == 'service.rs' and '::adapters::' in text:
            errors.append(str(relative) + ': service depends on a concrete adapter')
    expected = json.loads(CONTRACT.read_text())
    actual = snapshot(SOURCE, SOURCE / 'app/ipc.rs')
    for category in ['commands', 'serialized_types']:
        for name, value in expected[category].items():
            if actual[category].get(name) != value:
                errors.append(category + ' contract changed: ' + name)
        if category == 'commands':
            for name in actual[category].keys() - expected[category].keys():
                errors.append('unexpected registered command: ' + name)
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print(f"Backend boundaries OK; {len(expected['commands'])} commands and {len(expected['serialized_types'])} serialized types match the pre-refactor contracts.")
    return 0


if __name__ == '__main__':
    sys.exit(check())
