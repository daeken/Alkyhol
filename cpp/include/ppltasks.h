#pragma once
#include <future>
#include <utility>
namespace concurrency {
template<class T> class task {
    std::shared_future<T> value;
public:
    explicit task(std::future<T>&& future):value(std::move(future)){}
    T get()const {return value.get();}
    void wait()const {value.wait();}
};
template<class F> auto create_task(F&& function) {
    using T=decltype(function());
    return task<T>(std::async(std::launch::async,std::forward<F>(function)));
}
}
