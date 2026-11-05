if (this._textareaChangeTimer) {
      return;
    }
    const oldValue = this._textarea.value;
    this._textareaChangeTimer = window.setTimeout(() => {
      this._textareaChangeTimer = undefined;
      // Ignore if a composition has started since the timeout
      if (!this._isComposing) {
        const newValue = this._textarea.value;

        const diff = newValue.replace(oldValue, '');

        this._dataAlreadySent = diff;

        if (newValue.length > oldValue.length) {
          this._coreService.triggerDataEvent(diff, true);
        } else if (newValue.length < oldValue.length) {
          this._coreService.triggerDataEvent(`${C0.DEL}`, true);
        } else if ((newValue.length === oldValue.length) && (newValue !== oldValue)) {
          this._coreService.triggerDataEvent(newValue, true);
        }

      }
    }, 0);
