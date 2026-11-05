if (this._logLevel <= LogLevelEnum.DEBUG) {
      this._log(this._optionsService.options.logger?.debug.bind(this._optionsService.options.logger) ?? console.log, message, optionalParams);
    }
