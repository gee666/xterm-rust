
export function xtermBindLogger(logger, method, fallback, options) { return logger[method].bind(options === undefined ? logger : options.logger) ?? console[fallback]; }
export function xtermConsoleMethod(method) { return console[method]; }
export function xtermInvokeLog(method, message, parameters) { method.call(console, message, ...parameters); }
export function xtermLoggerTruthy(value) { return !!value; }
export function xtermLogPropertyKey(value) { return Reflect.ownKeys({[value]:0})[0]; }
