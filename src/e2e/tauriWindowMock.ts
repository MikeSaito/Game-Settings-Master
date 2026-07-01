export function getCurrentWindow() {
  return {
    listen: async () => () => {},
    onFocusChanged: async () => () => {},
    isFocused: async () => true,
  };
}
