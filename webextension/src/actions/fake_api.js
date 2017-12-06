function fakeUuid() {
  return Math.random().toString(36).substring(5);
}

function makeFakeTodo(name, dueDate, completionDate, labels = []) {
  return {uuid: fakeUuid(), name, dueDate, completionDate, labels};
}

const LABEL_P0 = { name: 'P0', color: 'rgb(184, 0, 0)' };
const LABEL_P1 = { name: 'P1', color: 'rgb(0, 77, 207)' };
const LABEL_SP = { name: 'Storage Prototype', color: 'rgb(83, 0, 235)' };
const LABEL_BL = { name: 'Backlog', color: 'rgb(0, 139, 2)' };

const memoryStore = {
  todos: [
    makeFakeTodo('Make Toodle WebExtension.', null, Date.now(), [LABEL_P1, LABEL_SP]),
    makeFakeTodo('Drink some hot chocolate.', Date.now(), null, [LABEL_P0]),
    makeFakeTodo('Double-click on a task name to edit.', Date.now(), null, [LABEL_BL])
  ],
  labels: [LABEL_P0, LABEL_P1, LABEL_SP, LABEL_BL]
};

function getTodoByUUID(uuid) {
  return memoryStore.todos.find(t => t.uuid === uuid);
}

function getLabelByName(name) {
  return memoryStore.labels.find(l => l.name === name);
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
  async todoChangeDueDate(uuid, dueDate) {
    const todo = getTodoByUUID(uuid);
    todo.dueDate = dueDate;
    return todo;
  },
  async todoChangeCompletionDate(uuid, completionDate) {
    const todo = getTodoByUUID(uuid);
    todo.completionDate = completionDate;
    return todo;
  },
  async todoAddLabel(uuid, labelName) {
    const todo = getTodoByUUID(uuid);
    todo.labels.push(getLabelByName(labelName));
    return Object.assign({}, todo);
  },
  async todoRemoveLabel(uuid, labelName) {
    const todo = getTodoByUUID(uuid);
    todo.labels = todo.labels.filter(l => l.name !== labelName);
    return Object.assign({}, todo);
  },
  async getLabels() {
    return memoryStore.labels.map(l => Object.assign({}, l));
  },
  async addLabel(name, color) {
    const newLabel = {name, color};
    memoryStore.labels.push(newLabel);
    return Object.assign({}, newLabel);
  },
  async removeLabel(labelName) {
    memoryStore.labels = memoryStore.labels.filter(l => l.name !== labelName);
    for (let todo of memoryStore.todos) {
      todo.labels = todo.labels.filter(l => l.name !== labelName);
    }
    return labelName;
  }
};

export default FakeApi;
