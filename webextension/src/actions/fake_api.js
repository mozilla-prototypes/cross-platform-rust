function fakeUuid() {
  return Math.random().toString(36).substring(5);
}

function makeFakeTodo(name, completionDate) {
  return {uuid: fakeUuid(), name, completionDate};
}

const memoryStore = {
  todos: [
    makeFakeTodo('Make Toodle WebExtension.', Date.now()),
    makeFakeTodo('Drink some hot chocolate.', null),
    makeFakeTodo('Double-click on a task name to edit.', null)
  ]
};

function getTodoByUUID(uuid) {
  return memoryStore.todos.find(t => t.uuid === uuid);
}

const FakeApi = {
  async createTodo(name) {
    const newTodo = makeFakeTodo(name);
    memoryStore.todos.push(newTodo);
    // We want a deep-copy!
    return Object.assign({}, newTodo);
  },
  async removeTodo(uuid) {
    memoryStore.todos = memoryStore.todos.filter(t => t.uuid !== uuid);
    return uuid;
  },
  async getTodos() {
    return memoryStore.todos.map(t => Object.assign({}, t));
  },
  async todoChangeName(uuid, newTodoName) {
    const todo = getTodoByUUID(uuid);
    todo.name = newTodoName;
    return todo;
  },
  async todoChangeCompletionDate(uuid, completionDate) {
    const todo = getTodoByUUID(uuid);
    todo.completionDate = completionDate;
    return todo;
  }
};

export default FakeApi;
