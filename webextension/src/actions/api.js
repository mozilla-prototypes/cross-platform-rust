const Api = {
  execute(request) {
    if (!this.port) {
      this.nextRequestId = 0;
      this.pendingRequests = new Map();
      this.port = browser.runtime.connectNative('toodlext');

      this.port.onMessage.addListener(response => {
        if (!this.pendingRequests.has(response.id)) {
          return;
        }
        let pendingRequest = this.pendingRequests.get(response.id);
        if (pendingRequest) {
          let { resolve, reject } = pendingRequest;
          (response.type == 'Ok' ? resolve : reject)(response.body);
          this.pendingRequests.delete(response.id);
        }
      });
    }
    return new Promise((resolve, reject) => {
      let requestId = ++this.nextRequestId;
      this.pendingRequests.set(requestId, { resolve, reject });
      this.port.postMessage({
        id: requestId,
        body: request,
      });
    });
  },

  async createTodo(name) {
    return this.execute({
      type: 'CreateTodo',
      name,
    });
  },
  async removeTodo(uuid) {
    let { uuid: removedUUID } = await this.execute({
      type: 'RemoveTodo',
      uuid,
    });
    return removedUUID;
  },
  async getTodos() {
    return this.execute({
      type: 'GetTodos',
    });
  },
  async todoChangeName(uuid, newTodoName) {
    return this.execute({
      type: 'TodoChangeName',
      name: newTodoName,
    });
  },
  async todoChangeCompletionDate(uuid, completionDate) {
    return this.execute({
      type: 'TodoChangeCompletionDate',
      uuid,
      completionDate,
    });
  }
};

export default Api;
