/**
 * JavaScript wrapper for consola-rs WASM bindings
 * 
 * This provides an ergonomic JavaScript API over the raw WASM bindings.
 */

import init, * as consola_wasm from './pkg/consola.js';

/**
 * Log levels (lower is more severe)
 */
export const LogLevel = {
  SILENT: -99,
  FATAL: 0,
  ERROR: 1,
  WARN: 2,
  LOG: 3,
  INFO: 4,
  SUCCESS: 5,
  DEBUG: 6,
  TRACE: 7,
  VERBOSE: 99
};

/**
 * Consola logger wrapper class
 */
export class ConsolaLogger {
  constructor(options = {}) {
    this.logger = null;
    this.level = options.level ?? LogLevel.INFO;
    this.autoFlush = options.autoFlush ?? true;
  }

  /**
   * Initialize the logger (async)
   * Must be called before using the logger
   */
  async init() {
    await init();
    this.logger = consola_wasm.create_logger_with_level(this.level);
    return this;
  }

  /**
   * Check if logger is initialized
   */
  isInitialized() {
    return this.logger !== null;
  }

  /**
   * Log an info message
   */
  info(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_info(this.logger, message);
  }

  /**
   * Log a warning message
   */
  warn(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_warn(this.logger, message);
  }

  /**
   * Log an error message
   * If the last argument is an Error object, it will be logged with stack trace
   */
  error(...args) {
    if (!this.logger) return;
    
    const lastArg = args[args.length - 1];
    if (lastArg instanceof Error) {
      const message = args.slice(0, -1).join(' ');
      consola_wasm.log_error_with_js_error(this.logger, message, lastArg);
    } else {
      const message = args.join(' ');
      consola_wasm.log_error(this.logger, message);
    }
  }

  /**
   * Log a debug message
   */
  debug(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_debug(this.logger, message);
  }

  /**
   * Log a trace message
   */
  trace(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_trace(this.logger, message);
  }

  /**
   * Log a success message
   */
  success(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_success(this.logger, message);
  }

  /**
   * Log a failure message
   */
  fail(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_fail(this.logger, message);
  }

  /**
   * Log a fatal error message
   */
  fatal(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_fatal(this.logger, message);
  }

  /**
   * Log a ready message
   */
  ready(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_ready(this.logger, message);
  }

  /**
   * Log a start message
   */
  start(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_start(this.logger, message);
  }

  /**
   * Log a box message (for multi-line content)
   */
  box(...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_box(this.logger, message);
  }

  /**
   * Log a message with a custom type
   */
  log(type, ...args) {
    if (!this.logger) return;
    const message = args.join(' ');
    consola_wasm.log_simple(this.logger, type, message);
  }

  /**
   * Set the log level
   */
  setLevel(level) {
    if (this.logger) {
      this.level = level;
      consola_wasm.set_level(this.logger, level);
    }
  }

  /**
   * Get the current log level
   */
  getLevel() {
    return this.level;
  }

  /**
   * Pause logging (messages will be buffered)
   */
  pause() {
    if (this.logger) {
      consola_wasm.pause(this.logger);
    }
  }

  /**
   * Resume logging (flush buffered messages)
   */
  resume() {
    if (this.logger) {
      consola_wasm.resume(this.logger);
    }
  }

  /**
   * Flush any buffered messages
   */
  flush() {
    if (this.logger) {
      consola_wasm.flush(this.logger);
    }
  }

  /**
   * Destroy the logger and free resources
   */
  destroy() {
    if (this.logger) {
      consola_wasm.free_logger(this.logger);
      this.logger = null;
    }
  }
}

/**
 * Create a default logger instance (convenience function)
 */
export async function createLogger(options = {}) {
  const logger = new ConsolaLogger(options);
  await logger.init();
  return logger;
}

/**
 * Create a global logger instance
 */
let globalLogger = null;

export async function initGlobalLogger(options = {}) {
  if (globalLogger) {
    globalLogger.destroy();
  }
  globalLogger = await createLogger(options);
  return globalLogger;
}

export function getGlobalLogger() {
  return globalLogger;
}

/**
 * Convenience functions using the global logger
 */
export const log = (...args) => globalLogger?.log(...args);
export const info = (...args) => globalLogger?.info(...args);
export const warn = (...args) => globalLogger?.warn(...args);
export const error = (...args) => globalLogger?.error(...args);
export const debug = (...args) => globalLogger?.debug(...args);
export const trace = (...args) => globalLogger?.trace(...args);
export const success = (...args) => globalLogger?.success(...args);
export const fail = (...args) => globalLogger?.fail(...args);
export const fatal = (...args) => globalLogger?.fatal(...args);
export const ready = (...args) => globalLogger?.ready(...args);
export const start = (...args) => globalLogger?.start(...args);
export const box = (...args) => globalLogger?.box(...args);
