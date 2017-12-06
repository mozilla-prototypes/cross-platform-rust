import React, { Component } from 'react';
import PropTypes from 'prop-types';
import * as Actions from '../actions';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Todo from './Todo';
import AddTodo from './AddTodo';
import './TodoList.css';

class TodoList extends Component {
  static propTypes = {
    todos: PropTypes.array.isRequired,
    allLabels: PropTypes.array.isRequired,
    populateLabels: PropTypes.func.isRequired,
    populateTodos: PropTypes.func.isRequired,
    todoChangeName: PropTypes.func.isRequired,
    todoAddLabel: PropTypes.func.isRequired,
    todoRemoveLabel: PropTypes.func.isRequired,
    todoChangeCompletionDate: PropTypes.func.isRequired,
    todoChangeDueDate: PropTypes.func.isRequired,
    removeTodo: PropTypes.func.isRequired
  }

  componentDidMount() {
    const { populateLabels, populateTodos } = this.props;
    populateLabels();
    populateTodos();
  }

  onTodoCompleted = (uuid, completed) => {
    this.props.todoChangeCompletionDate(uuid, completed ? Date.now() : null);
  }

  render() {
    const { todos, allLabels, todoAddLabel, todoRemoveLabel, todoChangeName,
      removeTodo, todoChangeDueDate, todoChangeCompletionDate } = this.props;
    return (
      <div className="todo-list">
        <h1>Todo List</h1>
        {todos.map(todo =>
          <Todo
            key={todo.uuid}
            {...todo}
            allLabels={allLabels}
            onRemoveTodo={() => removeTodo(todo.uuid)}
            onTodoLabelAdded={todoAddLabel}
            onTodoLabelRemoved={todoRemoveLabel}
            onTodoNameChanged={todoChangeName}
            onTodoChangeCompletionDate={todoChangeCompletionDate}
            onTodoChangeDueDate={todoChangeDueDate}
            onTodoCompleted={this.onTodoCompleted}
          />
        )}
        <div className="todo-wrapper"><AddTodo /></div>
      </div>
    );
  }
}

const mapStateToProps = (state) => ({
  todos: state.todos,
  allLabels: state.labels
});

const mapDispatchToProps = (dispatch) => {
  const { populateLabels, populateTodos, todoChangeName, todoAddLabel,
    todoRemoveLabel, todoChangeCompletionDate, todoChangeDueDate,
    removeTodo } = Actions;
  return {
    ...bindActionCreators({ populateLabels, populateTodos,
      todoChangeCompletionDate, todoChangeDueDate,
      todoAddLabel, todoRemoveLabel, todoChangeName,
      removeTodo
    }, dispatch)
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TodoList);
