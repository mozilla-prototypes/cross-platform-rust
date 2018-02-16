import Api from './api';

export const populateTodos = () => ({
  type: 'POPULATE_TODOS',
  payload: Api.getTodos()
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

export const todoChangeCompletionDate = (uuid, completionDate) => ({
  type: 'TODO_CHANGE_COMPLETION_DATE',
  payload: Api.todoChangeCompletionDate(uuid, completionDate)
});
