// Prevents all events to pty process if stdin is disabled
    if (this._optionsService.rawOptions.disableStdin) {
      return;
    }

    // Input is being sent to the terminal, the terminal should focus the prompt.
    const buffer = this._bufferService.buffer;
    if (wasUserInput && this._optionsService.rawOptions.scrollOnUserInput && buffer.ybase !== buffer.ydisp) {
      this._onRequestScrollToBottom.fire();
    }

    // Fire onUserInput so listeners can react as well (eg. clear selection)
    if (wasUserInput) {
      this._onUserInput.fire();
    }

    // Fire onData API
    this._logService.debug(`sending data "${data}"`);
    this._logService.trace(`sending data (codes)`, () => data.split('').map(e => e.charCodeAt(0)));
    this._onData.fire(data);
