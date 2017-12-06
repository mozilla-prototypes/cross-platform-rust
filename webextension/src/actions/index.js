import FakeApi from './fake_api';

export const populateTodos = () => ({
  type: 'POPULATE_TODOS',
  payload: FakeApi.getTodos()
});

export const populateLabels = () => ({
  type: 'POPULATE_LABELS',
  payload: FakeApi.getLabels()
});

export const addTodo = (text) => ({
  type: 'ADD_TODO',
  payload: FakeApi.createTodo(text)
});

export const removeTodo = (uuid) => ({
  type: 'REMOVE_TODO',
  payload: FakeApi.removeTodo(uuid)
});

export const todoChangeName = (uuid, newTodoName) => ({
  type: 'TODO_CHANGE_NAME',
  payload: FakeApi.todoChangeName(uuid, newTodoName)
});

export const todoChangeDueDate = (uuid, dueDate) => ({
  type: 'TODO_CHANGE_DUE_DATE',
  payload: FakeApi.todoChangeDueDate(uuid, dueDate)
});

export const todoChangeCompletionDate = (uuid, completionDate) => ({
  type: 'TODO_CHANGE_COMPLETION_DATE',
  payload: FakeApi.todoChangeCompletionDate(uuid, completionDate)
});

export const todoAddLabel = (uuid, labelName) => ({
  type: 'TODO_ADD_LABEL',
  payload: FakeApi.todoAddLabel(uuid, labelName)
});

export const todoRemoveLabel = (uuid, labelName) => ({
  type: 'TODO_REMOVE_LABEL',
  payload: FakeApi.todoRemoveLabel(uuid, labelName)
});

export const addLabel = (labelName, color) => ({
  type: 'ADD_LABEL',
  payload: FakeApi.addLabel(labelName, color)
});

export const removeLabel = (labelName) => ({
  type: 'REMOVE_LABEL',
  payload: FakeApi.removeLabel(labelName)
});
