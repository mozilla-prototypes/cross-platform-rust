import { combineReducers } from 'redux';
import todos from './todos';
import labels from './labels';

const todoApp = combineReducers({
  labels,
  todos
});

export default todoApp;
