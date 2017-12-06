const labels = (state = [], action) => {
  switch (action.type) {
  case 'POPULATE_LABELS_FULFILLED':
    return action.payload;
  case 'ADD_LABEL_FULFILLED':
    return [...state, action.payload];
  case 'REMOVE_LABEL_FULFILLED':
    return state.filter(l => l.name !== action.payload);
  default:
    return state;
  }
};

export default labels;
