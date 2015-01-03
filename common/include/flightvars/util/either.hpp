/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_UTIL_EITHER_H
#define FLIGHTVARS_UTIL_EITHER_H

#include <memory>

#include <flightvars/util/exception.hpp>

namespace flightvars { namespace util {

FV_DECL_EXCEPTION(either_error);

/** 
 * Either is an object that stores one value of two possible types. 
 * `either<T, U>` stores either a value of T or a value of U. The value of T is known as the
 * left side, while the U value is the right side. On `either` constructor, the value to fill
 * the object is provided (only left or right is allowed, never both). Then it is possible to 
 * check how `either` is populated with `has_left()` and `has_right()` functions. The wrapped 
 * value can be extracted with `left()` and `right()`.  
 */
template <class T, class U>
class either {
public:

    static_assert(!std::is_void<T>::value && !std::is_void<U>::value,
            "cannot instantiate an either object with a void type");

    /** Create a new either object with nor left neither right defined. */
    either() = default;

    /** Create a new either object with left defined by copy. */
    either(const T& left) : _left(new T(left)), _right(nullptr) {}

    /** Create a new either object with left defined by move. */
    either(T&& left) : _left(new T(std::move(left))), _right(nullptr) {}

    /** Create a new either object with right defined by copy. */
    either(const U& right) : _left(nullptr), _right(new U(right)) {}

    /** Create a new either object with right defined by move. */
    either(U&& right) : _left(nullptr), _right(new U(std::move(right))) {}

    either(const either& other) :
        _left(other.has_left() ? new T(other.left()) : nullptr),
        _right(other.has_right() ? new U(other.right()) : nullptr) {}

    either(either&& other) = default;

    either& operator = (const either& other) {
        _left.reset(other.has_left() ? new T(other.left()) : nullptr);
        _right.reset(other.has_right() ? new U(other.right()) : nullptr);        
        return *this;
    }

    either& operator = (either&& other) = default;

    /** Reset to none value. */
    void reset() {
        _left.reset();
        _right.reset();
    }

    /** Reset to a left value. */
    void reset(const T& left) {
        _left.reset(new T(left));
        _right.reset();
    }

    /** Reset to a left value. */
    void reset(T&& left) {
        _left.reset(new T(std::move(left)));
        _right.reset();
    }

    /** Reset to a left value. */
    void reset(const U& right) {
        _left.reset();
        _right.reset(new U(right));
    }

    /** Reset to a left value. */
    void reset(U&& right) {
        _left.reset();
        _right.reset(new U(std::move(right)));
    }

    /** True if the either has a state, false otherwise. */
    bool valid() const { return has_left() || has_right(); }

    /** True if populated with left value, false otherwise. */
    bool has_left() const { return !!_left; }

    /** True if populated with right value, false otherwise. */
    bool has_right() const { return !!_right; }

    /** Obtain the left item, or throw `either_undefined` if not populated with left value. */
    const T& left() const {
        if (has_left()) { return *_left; }
        else { throw either_error("cannot get undefined left part of either"); }
    }

    /** Obtain the left item, or throw `either_undefined` if not populated with left value. */
    T& left() {
        if (has_left()) { return *_left; }
        else { throw either_error("cannot get undefined left part of either"); }
    }

    /** Extract the left item, or throw `either_undefined` if not populated with left value. */
    T extract_left() {
        if (has_left()) {
            T r = std::move(*_left);
            reset();
            return r;
        }
        else { throw either_error("cannot get undefined left part of either"); }
    }

    /** Obtain the right item, or throw `either_undefined` if not populated with right value. */
    const U& right() const {
        if (has_right()) { return *_right; }
        else { throw either_error("cannot get undefined right part of either"); }
    }

    /** Obtain the right item, or throw `either_undefined` if not populated with right value. */
    U& right() {
        if (has_right()) { return *_right; }
        else { throw either_error("cannot get undefined right part of either"); }
    }

    /** Extract the left item, or throw `either_undefined` if not populated with right value. */
    U extract_right() {
        if (has_right()) {
            U r = std::move(*_right);
            reset();
            return r;
        }
        else { throw either_error("cannot get undefined left part of either"); }
    }

private:

    std::unique_ptr<T> _left;
    std::unique_ptr<U> _right;
};

}}

#endif

