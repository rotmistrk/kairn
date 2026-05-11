# Bug: Wide char rendering artifacts

Stale cells appear after panel resize and in shell window when wide characters are present.

The diff flush fix (comparing width field) partially solved it but the issue persists.

Likely causes:
1. Wide char at column boundary: char occupies cols N and N+1, but panel ends at N — second half leaks
2. Shell output with wide chars: TermBuf renders them but the diff doesn't properly track the 2-cell span
3. When a wide char moves position, both the old AND new positions need updating

Needs: reproduce with specific test case, inspect Surface cells vs previous buffer cells at the artifact location.
