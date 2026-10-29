// Camera work is completion-driven: one frame in flight, no pending frame queue.
export function scanDelay(workMs, sessionMs) {
  const duty = sessionMs >= 15000 ? 0.15 : 0.30;
  return Math.max(90, Math.max(0, workMs) * (1 / duty - 1));
}
export function scanSize(width, height, attempt, confirming = false) {
  // A higher-resolution retry preserves small/dense codes; cheap frames dominate.
  const limit = confirming || attempt % 4 === 3 ? 960 : 640;
  const scale = Math.min(1, limit / Math.max(width, height));
  return {width: Math.max(32, Math.round(width * scale)), height: Math.max(32, Math.round(height * scale))};
}
