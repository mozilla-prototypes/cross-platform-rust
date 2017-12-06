import React, { Component } from 'react';
import onClickOutside from 'react-onclickoutside';
import PropTypes from 'prop-types';
import './TodoDropdownMenu.css';

class TodoDropdownMenu extends Component {
  static propTypes = {
    children: PropTypes.object.isRequired,
    onCloseDropdown: PropTypes.func.isRequired
  }

  handleClickOutside = () => {
    this.props.onCloseDropdown();
  };

  render() {
    return (
      <div className="todo-dropdown-menu">
        {this.props.children}
      </div>
    );
  }
}

export default onClickOutside(TodoDropdownMenu);
