if (this._logLevel <= LogLevelEnum.TRACE) {
      this._log(this._optionsService.options.logger?.trace.bind(this._optionsService.options.logger) ?? console.log, message, optionalParams);
    }
