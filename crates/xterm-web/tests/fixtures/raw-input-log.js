this._evalLazyOptionalParams(optionalParams);
    type.call(console, (this._optionsService.options.logger ? '' : LOG_PREFIX) + message, ...optionalParams);
