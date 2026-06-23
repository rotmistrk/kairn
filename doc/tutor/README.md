# Interactive Coding Tutor

An MCP-powered coding tutor that guides users through structured lessons.

## Usage

```
kiro --agent=tutor
```

## Lesson Format

Lessons are markdown files with special markers:

- `STEP:` — A task for the user to complete
- `CHECK:` — A condition to verify (glob pattern or regex on file content)
- `HINT:` — Help text shown when the user is stuck

## Example

See `lessons/01-rust-iterators.md` for a sample lesson.

## How It Works

1. The tutor agent reads the lesson file
2. Presents one STEP at a time
3. After the user makes changes, verifies CHECKs
4. Provides HINTs on request
5. Advances to the next step on success
