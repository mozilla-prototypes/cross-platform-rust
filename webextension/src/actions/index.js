import Api from './api';

export const populateTodos = () => ({
  type: 'POPULATE_TODOS',
  payload: Api.getTodos()
});

export const populateLabels = () => ({
  type: 'POPULATE_LABELS',
  payload: Api.getLabels()
});

export const addTodo = (text) => ({
  type: 'ADD_TODO',
  payload: Api.createTodo(text)
});

export const removeTodo = (uuid) => ({
  type: 'REMOVE_TODO',
  payload: Api.removeTodo(uuid)
});

export const todoChangeName = (uuid, newTodoName) => ({
  type: 'TODO_CHANGE_NAME',
  payload: Api.todoChangeName(uuid, newTodoName)
});

export const todoChangeDueDate = (uuid, dueDate) => ({
  type: 'TODO_CHANGE_DUE_DATE',
  payload: Api.todoChangeDueDate(uuid, dueDate)
});

export const todoChangeCompletionDate = (uuid, completionDate) => ({
  type: 'TODO_CHANGE_COMPLETION_DATE',
  payload: Api.todoChangeCompletionDate(uuid, completionDate)
});

export const todoAddLabel = (uuid, labelName) => ({
  type: 'TODO_ADD_LABEL',
  payload: Api.todoAddLabel(uuid, labelName)
});

export const todoRemoveLabel = (uuid, labelName) => ({
  type: 'TODO_REMOVE_LABEL',
  payload: Api.todoRemoveLabel(uuid, labelName)
});

export const addLabel = (labelName, color) => ({
  type: 'ADD_LABEL',
  payload: Api.addLabel(labelName, color)
});

export const removeLabel = (labelName) => ({
  type: 'REMOVE_LABEL',
  payload: Api.removeLabel(labelName)
});
