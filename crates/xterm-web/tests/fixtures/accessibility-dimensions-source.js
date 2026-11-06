// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Generated unchanged method bodies. Test-only, never imported by the library.
// AccessibilityManager.ts SHA-256: 6519cc3ee85382a7da9b42a112010044f78f3a1cd4d02fe4fcb37bea36f526ec
export class SourceDimensions {
_refreshRowsDimensions() {
    if (!this._renderService.dimensions.css.cell.height) {
      return;
    }
    Object.assign(this._accessibilityContainer.style, {
      width: `${this._renderService.dimensions.css.canvas.width}px`,
      fontSize: `${this._terminal.options.fontSize}px`
    });
    if (this._rowElements.length !== this._terminal.rows) {
      this._handleResize(this._terminal.rows);
    }
    for (let i = 0; i < this._terminal.rows; i++) {
      this._refreshRowDimensions(this._rowElements[i]);
      this._alignRowWidth(this._rowElements[i]);
    }
  }
_refreshRowDimensions(element) {
    element.style.height = `${this._renderService.dimensions.css.cell.height}px`;
  }
_alignRowWidth(element) {
    element.style.transform = '';
    const width = element.getBoundingClientRect().width;
    const lastColumn = this._rowColumns.get(element)?.slice(-1)?.[0];
    if (!lastColumn) {
      return;
    }
    const targetWidth = lastColumn * this._renderService.dimensions.css.cell.width;
    element.style.transform = `scaleX(${targetWidth / width})`;
  }
}
