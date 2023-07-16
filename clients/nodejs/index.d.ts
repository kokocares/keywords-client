/**
Check if text matches keywords to detect high-risk search queries

@example
```
import { match } from '@kokocares/keywords';

const matches = match('text');

console.log(matches);
// True
```
*/
export function match(text: string, filters?: string | null, version?: string | null): Boolean;
