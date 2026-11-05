this._isComposing = true;
    // It's important to use the selection here instead of textarea length to avoid conflicts with
    // screen reader mode
    const start = this._textarea.selectionStart ?? this._textarea.value.length;
    const end = this._textarea.selectionEnd ?? start;
    this._compositionPosition.start = Math.min(start, end);
    this._compositionPosition.end = Math.max(start, end);
    this._compositionSuffix = this._textarea.value.substring(this._compositionPosition.end);
    this._compositionView.textContent = '';
    this._dataAlreadySent = '';
    this._compositionView.classList.add('active');
