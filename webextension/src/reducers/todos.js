const todos = (state = [], action) => {
  switch (action.type) {
  case 'POPULATE_TODOS_FULFILLED':
    return action.payload;
  case 'ADD_TODO_FULFILLED':
    return [
      ...state,
      action.payload
    ];
  case 'REMOVE_TODO_FULFILLED':
    return state.filter(t => t.uuid !== action.payload);
  case 'TODO_ADD_LABEL_FULFILLED':
  case 'TODO_REMOVE_LABEL_FULFILLED':
  case 'TODO_CHANGE_NAME_FULFILLED':
  case 'TODO_CHANGE_DUE_DATE_FULFILLED':
  case 'TODO_CHANGE_COMPLETION_DATE_FULFILLED':
    // Update the TODO we have with the one we just received
    return state.map(todo => {
      if (todo.uuid !== action.payload.uuid) {
        return todo;
      }
      return action.payload;
    });
  case 'REMOVE_LABEL_FULFILLED':
    return state.map(t => {
      return {
        ...t,
        labels: t.labels.filter(l => l.name !== action.payload)
      };
    });
  default:
    return state;
  }
};

export default todos;
