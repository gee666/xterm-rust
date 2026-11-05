// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Capture the browser's synchronous error event, not dispatchEvent's boolean.
return (element, kind, marker) => {
  const errors = [];
  const onError = event => { errors.push(event.error); event.preventDefault(); };
  window.addEventListener('error', onError);
  try { element.dispatchEvent(new MouseEvent(kind)); }
  finally { window.removeEventListener('error', onError); }
  return [errors.length, errors.length === 1 && errors[0] === marker];
};
