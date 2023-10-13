const b = {};
const c = { b };
const a = { b, c };
const d = [a, a, b, c];

// 0->1
// 1->2
// 1->3
// 0->2
// 0->3
// 3->2

// 2
// 3
// 1
// 0
