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
    populateTodos: PropTypes.func.isRequired,
    todoChangeName: PropTypes.func.isRequired,
    todoChangeCompletionDate: PropTypes.func.isRequired,
    removeTodo: PropTypes.func.isRequired
  }

  componentDidMount() {
    const { populateTodos } = this.props;
    populateTodos();
  }

  onTodoCompleted = (uuid, completed) => {
    this.props.todoChangeCompletionDate(uuid, completed ? Date.now() : null);
  }

  render() {
    const { todos, todoChangeName,
      removeTodo, todoChangeCompletionDate } = this.props;
    return (
      <div className="todo-list">
        <h1>Todo List</h1>
        {todos.map(todo =>
          <Todo
            key={todo.uuid}
            {...todo}
            onRemoveTodo={() => removeTodo(todo.uuid)}
            onTodoNameChanged={todoChangeName}
            onTodoChangeCompletionDate={todoChangeCompletionDate}
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
});

const mapDispatchToProps = (dispatch) => {
  const { populateTodos, todoChangeName,
    todoChangeCompletionDate,
    removeTodo } = Actions;
  return {
    ...bindActionCreators({ populateTodos,
      todoChangeCompletionDate,
      todoChangeName,
      removeTodo
    }, dispatch)
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TodoList);
