this._compositionView.classList.remove('active');
    this._isComposing = false;

    if (!waitForPropagation) {
      // Cancel any delayed composition send requests and send the input immediately.
      this._isSendingComposition = false;
      const input = this._textarea.value.substring(this._compositionPosition.start, this._compositionPosition.end);
      this._coreService.triggerDataEvent(input, true);
    } else {
      // Make a deep copy of the composition position here as a new compositionstart event may
      // fire before the setTimeout executes.
      const currentCompositionPosition = {
        start: this._compositionPosition.start,
        end: this._compositionPosition.end
      };
      const currentCompositionSuffix = this._compositionSuffix;

      // Since composition* events happen before the changes take place in the textarea on most
      // browsers, use a setTimeout with 0ms time to allow the native compositionend event to
      // complete. This ensures the correct character is retrieved.
      // This solution was used because:
      // - The compositionend event's data property is unreliable, at least on Chromium
      // - The last compositionupdate event's data property does not always accurately describe
      //   the character, a counter example being Korean where an ending consonsant can move to
      //   the following character if the following input is a vowel.
      this._isSendingComposition = true;
      setTimeout(() => {
        // Ensure that the input has not already been sent
        if (this._isSendingComposition) {
          this._isSendingComposition = false;
          let input;
          // Add length of data already sent due to keydown event,
          // otherwise input characters can be duplicated. (Issue #3191)
          currentCompositionPosition.start += this._dataAlreadySent.length;
          if (this._isComposing) {
            // Use the start position of the new composition to get the string
            // if a new composition has started.
            input = this._textarea.value.substring(currentCompositionPosition.start, this._compositionPosition.start);
          } else {
            // Keep support for non-composition characters typed immediately after composition end
            // while avoiding re-sending the trailing text that was already present
            // before composition started.
            const value = this._textarea.value;
            const valueEnd = currentCompositionSuffix.length > 0 && value.endsWith(currentCompositionSuffix)
              ? value.length - currentCompositionSuffix.length
              : value.length;
            input = value.substring(currentCompositionPosition.start, Math.max(currentCompositionPosition.start, valueEnd));
          }
          if (input.length > 0) {
            this._coreService.triggerDataEvent(input, true);
          }
        }
      }, 0);
    }
