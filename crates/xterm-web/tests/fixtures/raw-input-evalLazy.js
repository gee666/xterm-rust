for (let i = 0; i < optionalParams.length; i++) {
      if (typeof optionalParams[i] === 'function') {
        optionalParams[i] = optionalParams[i]();
      }
    }
