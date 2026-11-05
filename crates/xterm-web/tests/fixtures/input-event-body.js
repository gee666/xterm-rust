
    // Only support emoji IMEs when screen reader mode is disabled as the event must bubble up to
    // support reading out character input which can doubling up input characters
    // Based on these event traces: https://github.com/xtermjs/xterm.js/issues/3679
    if (ev.data && ev.inputType === 'insertText' && (!ev.composed || !this._keyDownSeen) && !this.optionsService.rawOptions.screenReaderMode) {
      if (this._keyPressHandled) {
        return false;
      }

      // The key was handled so clear the dead key state, otherwise certain keystrokes like arrow
      // keys could be ignored
      this._unprocessedDeadKey = false;

      const text = ev.data;
      this.coreService.triggerDataEvent(text, true);
      return true;
    }

    return false;
  